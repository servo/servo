/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie::Cookie;
use fetch::methods::{should_be_blocked_due_to_bad_port, should_be_blocked_due_to_nosniff};
use hosts::replace_host;
use http_loader::{HttpState, is_redirect_status, set_default_accept};
use http_loader::{set_default_accept_language, set_request_cookies};
use hyper::buffer::BufReader;
use hyper::header::{CacheControl, CacheDirective, Connection, ConnectionOption};
use hyper::header::{Headers, Host, SetCookie, Pragma, Protocol, ProtocolName, Upgrade};
use hyper::http::h1::{LINE_ENDING, parse_response};
use hyper::method::Method;
use hyper::net::HttpStream;
use hyper::status::StatusCode;
use hyper::version::HttpVersion;
use net_traits::{CookieSource, MessageData, NetworkError, WebSocketCommunicate, WebSocketConnectData};
use net_traits::{WebSocketDomAction, WebSocketNetworkEvent};
use net_traits::request::{Destination, Type};
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use url::Position;
use websocket::{Message, Receiver as WSReceiver, Sender as WSSender};
use websocket::header::{Origin, WebSocketAccept, WebSocketKey, WebSocketProtocol, WebSocketVersion};
use websocket::message::Type as MessageType;
use websocket::receiver::Receiver;
use websocket::sender::Sender;

pub fn init(connect: WebSocketCommunicate,
            connect_data: WebSocketConnectData,
            http_state: Arc<HttpState>) {
    thread::Builder::new().name(format!("WebSocket connection to {}", connect_data.resource_url)).spawn(move || {
        let channel = establish_a_websocket_connection(connect_data.resource_url,
                                                       connect_data.origin,
                                                       connect_data.protocols,
                                                       &http_state);
        let (ws_sender, mut receiver) = match channel {
            Ok((protocol_in_use, sender, receiver)) => {
                let _ = connect.event_sender.send(WebSocketNetworkEvent::ConnectionEstablished { protocol_in_use });
                (sender, receiver)
            },
            Err(e) => {
                debug!("Failed to establish a WebSocket connection: {:?}", e);
                let _ = connect.event_sender.send(WebSocketNetworkEvent::Fail);
                return;
            }

        };

        let initiated_close = Arc::new(AtomicBool::new(false));
        let ws_sender = Arc::new(Mutex::new(ws_sender));

        let initiated_close_incoming = initiated_close.clone();
        let ws_sender_incoming = ws_sender.clone();
        let resource_event_sender = connect.event_sender;
        thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: Message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        debug!("Error receiving incoming WebSocket message: {:?}", e);
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Fail);
                        break;
                    }
                };
                let message = match message.opcode {
                    MessageType::Text => MessageData::Text(String::from_utf8_lossy(&message.payload).into_owned()),
                    MessageType::Binary => MessageData::Binary(message.payload.into_owned()),
                    MessageType::Ping => {
                        let pong = Message::pong(message.payload);
                        ws_sender_incoming.lock().unwrap().send_message(&pong).unwrap();
                        continue;
                    },
                    MessageType::Pong => continue,
                    MessageType::Close => {
                        if !initiated_close_incoming.fetch_or(true, Ordering::SeqCst) {
                            ws_sender_incoming.lock().unwrap().send_message(&message).unwrap();
                        }
                        let code = message.cd_status_code;
                        let reason = String::from_utf8_lossy(&message.payload).into_owned();
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Close(code, reason));
                        break;
                    },
                };
                let _ = resource_event_sender.send(WebSocketNetworkEvent::MessageReceived(message));
            }
        });

        while let Ok(dom_action) = connect.action_receiver.recv() {
            match dom_action {
                WebSocketDomAction::SendMessage(MessageData::Text(data)) => {
                    ws_sender.lock().unwrap().send_message(&Message::text(data)).unwrap();
                },
                WebSocketDomAction::SendMessage(MessageData::Binary(data)) => {
                    ws_sender.lock().unwrap().send_message(&Message::binary(data)).unwrap();
                },
                WebSocketDomAction::Close(code, reason) => {
                    if !initiated_close.fetch_or(true, Ordering::SeqCst) {
                        let message = match code {
                            Some(code) => Message::close_because(code, reason.unwrap_or("".to_owned())),
                            None => Message::close()
                        };
                        ws_sender.lock().unwrap().send_message(&message).unwrap();
                    }
                },
            }
        }
    }).expect("Thread spawning failed");
}

type Stream = HttpStream;

// https://fetch.spec.whatwg.org/#concept-websocket-connection-obtain
fn obtain_a_websocket_connection(url: &ServoUrl) -> Result<Stream, NetworkError> {
    // Step 1.
    let host = url.host_str().unwrap();

    // Step 2.
    let port = url.port_or_known_default().unwrap();

    // Step 3.
    // We did not replace the scheme by "http" or "https" in step 1 of
    // establish_a_websocket_connection.
    let secure = match url.scheme() {
        "ws" => false,
        "wss" => true,
        _ => panic!("URL's scheme should be ws or wss"),
    };

    if secure {
        return Err(NetworkError::Internal("WSS is disabled for now.".into()));
    }

    // Steps 4-5.
    let host = replace_host(host);
    let tcp_stream = TcpStream::connect((&*host, port)).map_err(|e| {
        NetworkError::Internal(format!("Could not connect to host: {}", e))
    })?;
    Ok(HttpStream(tcp_stream))
}

// https://fetch.spec.whatwg.org/#concept-websocket-establish
fn establish_a_websocket_connection(resource_url: ServoUrl,
                                    origin: String,
                                    protocols: Vec<String>,
                                    http_state: &HttpState)
                                    -> Result<(Option<String>,
                                               Sender<Stream>,
                                               Receiver<Stream>),
                                              NetworkError> {
    // Steps 1 is not really applicable here, given we don't exactly go
    // through the same infrastructure as the Fetch spec.

    // Step 2, slimmed down because we don't go through the whole Fetch infra.
    let mut headers = Headers::new();

    // Step 3.
    headers.set(Upgrade(vec![Protocol::new(ProtocolName::WebSocket, None)]));

    // Step 4.
    headers.set(Connection(vec![ConnectionOption::ConnectionHeader("upgrade".into())]));

    // Step 5.
    let key_value = WebSocketKey::new();

    // Step 6.
    headers.set(key_value);

    // Step 7.
    headers.set(WebSocketVersion::WebSocket13);

    // Step 8.
    if !protocols.is_empty() {
        headers.set(WebSocketProtocol(protocols.clone()));
    }

    // Steps 9-10.
    // TODO: handle permessage-deflate extension.

    // Step 11 and network error check from step 12.
    let response = fetch(resource_url, origin, headers, http_state)?;

    // Step 12, the status code check.
    if response.status != StatusCode::SwitchingProtocols {
        return Err(NetworkError::Internal("Response's status should be 101.".into()));
    }

    // Step 13.
    if !protocols.is_empty() {
        if response.headers.get::<WebSocketProtocol>().map_or(true, |protocols| protocols.is_empty()) {
            return Err(NetworkError::Internal(
                "Response's Sec-WebSocket-Protocol header is missing, malformed or empty.".into()));
        }
    }

    // Step 14.2.
    let upgrade_header = response.headers.get::<Upgrade>().ok_or_else(|| {
        NetworkError::Internal("Response should have an Upgrade header.".into())
    })?;
    if upgrade_header.len() != 1 {
        return Err(NetworkError::Internal("Response's Upgrade header should have only one value.".into()));
    }
    if upgrade_header[0].name != ProtocolName::WebSocket {
        return Err(NetworkError::Internal("Response's Upgrade header value should be \"websocket\".".into()));
    }

    // Step 14.3.
    let connection_header = response.headers.get::<Connection>().ok_or_else(|| {
        NetworkError::Internal("Response should have a Connection header.".into())
    })?;
    let connection_includes_upgrade = connection_header.iter().any(|option| {
        match *option {
            ConnectionOption::ConnectionHeader(ref option) => *option == "upgrade",
            _ => false,
        }
    });
    if !connection_includes_upgrade {
        return Err(NetworkError::Internal("Response's Connection header value should include \"upgrade\".".into()));
    }

    // Step 14.4.
    let accept_header = response.headers.get::<WebSocketAccept>().ok_or_else(|| {
        NetworkError::Internal("Response should have a Sec-Websocket-Accept header.".into())
    })?;
    if *accept_header != WebSocketAccept::new(&key_value) {
        return Err(NetworkError::Internal(
            "Response's Sec-WebSocket-Accept header value did not match the sent key.".into()));
    }

    // Step 14.5.
    // TODO: handle permessage-deflate extension.
    // We don't support any extension, so we fail at the mere presence of
    // a Sec-WebSocket-Extensions header.
    if response.headers.get_raw("Sec-WebSocket-Extensions").is_some() {
        return Err(NetworkError::Internal(
            "Response's Sec-WebSocket-Extensions header value included unsupported extensions.".into()));
    }

    // Step 14.6.
    let protocol_in_use = if let Some(response_protocols) = response.headers.get::<WebSocketProtocol>() {
        for replied in &**response_protocols {
            if !protocols.iter().any(|requested| requested.eq_ignore_ascii_case(replied)) {
                return Err(NetworkError::Internal(
                    "Response's Sec-WebSocket-Protocols contain values that were not requested.".into()));
            }
        }
        response_protocols.first().cloned()
    } else {
        None
    };

    let sender = Sender::new(response.writer, true);
    let receiver = Receiver::new(response.reader, false);
    Ok((protocol_in_use, sender, receiver))
}

struct Response {
    status: StatusCode,
    headers: Headers,
    reader: BufReader<Stream>,
    writer: Stream,
}

// https://fetch.spec.whatwg.org/#concept-fetch
fn fetch(url: ServoUrl,
         origin: String,
         mut headers: Headers,
         http_state: &HttpState)
         -> Result<Response, NetworkError> {
    // Step 1.
    // TODO: handle request's window.

    // Step 2.
    // TODO: handle request's origin.

    // Step 3.
    set_default_accept(Type::None, Destination::None, &mut headers);

    // Step 4.
    set_default_accept_language(&mut headers);

    // Step 5.
    // TODO: handle request's priority.

    // Step 6.
    // Not applicable: not a navigation request.

    // Step 7.
    // We know this is a subresource request.
    {
        // Step 7.1.
        // Not applicable: client hints list is empty.

        // Steps 7.2-3.
        // TODO: handle fetch groups.
    }

    // Step 8.
    main_fetch(url, origin, headers, http_state)
}

// https://fetch.spec.whatwg.org/#concept-main-fetch
fn main_fetch(url: ServoUrl,
              origin: String,
              mut headers: Headers,
              http_state: &HttpState)
              -> Result<Response, NetworkError> {
    // Step 1.
    let mut response = None;

    // Step 2.
    // Not applicable: request’s local-URLs-only flag is unset.

    // Step 3.
    // TODO: handle content security policy violations.

    // Step 4.
    // TODO: handle upgrade to a potentially secure URL.

    // Step 5.
    if should_be_blocked_due_to_bad_port(&url) {
        response = Some(Err(NetworkError::Internal("Request should be blocked due to bad port.".into())));
    }
    // TODO: handle blocking as mixed content.
    // TODO: handle blocking by content security policy.

    // Steps 6-8.
    // TODO: handle request's referrer policy.

    // Step 9.
    // Not applicable: request's current URL's scheme is not "ftp".

    // Step 10.
    // TODO: handle known HSTS host domain.

    // Step 11.
    // Not applicable: request's synchronous flag is set.

    // Step 12.
    let mut response = response.unwrap_or_else(|| {
        // We must run the first sequence of substeps, given request's mode
        // is "websocket".

        // Step 12.1.
        // Not applicable: the response is never exposed to the Web so it
        // doesn't need to be filtered at all.

        // Step 12.2.
        scheme_fetch(&url, origin, &mut headers, http_state)
    });

    // Step 13.
    // Not applicable: recursive flag is unset.

    // Step 14.
    // Not applicable: the response is never exposed to the Web so it doesn't
    // need to be filtered at all.

    // Steps 15-16.
    // Not applicable: no need to maintain an internal response.

    // Step 17.
    if response.is_ok() {
        // TODO: handle blocking as mixed content.
        // TODO: handle blocking by content security policy.
        // Not applicable: blocking due to MIME type matters only for scripts.
        if should_be_blocked_due_to_nosniff(Type::None, &headers) {
            response = Err(NetworkError::Internal("Request should be blocked due to nosniff.".into()));
        }
    }

    // Step 18.
    // Not applicable: we don't care about the body at all.

    // Step 19.
    // Not applicable: request's integrity metadata is the empty string.

    // Step 20.
    // TODO: wait for response's body here, maybe?
    response
}

// https://fetch.spec.whatwg.org/#concept-scheme-fetch
fn scheme_fetch(url: &ServoUrl,
               origin: String,
               headers: &mut Headers,
               http_state: &HttpState)
               -> Result<Response, NetworkError> {
    // In the case of a WebSocket request, HTTP fetch is always used.
    http_fetch(url, origin, headers, http_state)
}

// https://fetch.spec.whatwg.org/#concept-http-fetch
fn http_fetch(url: &ServoUrl,
              origin: String,
              headers: &mut Headers,
              http_state: &HttpState)
              -> Result<Response, NetworkError> {
    // Step 1.
    // Not applicable: with step 3 being useless here, this one is too.

    // Step 2.
    // Not applicable: we don't need to maintain an internal response.

    // Step 3.
    // Not applicable: request's service-workers mode is "none".

    // Step 4.
    // There cannot be a response yet at this point.
    let mut response = {
        // Step 4.1.
        // Not applicable: CORS-preflight flag is unset.

        // Step 4.2.
        // Not applicable: request's redirect mode is "error".

        // Step 4.3.
        let response = http_network_or_cache_fetch(url, origin, headers, http_state);

        // Step 4.4.
        // Not applicable: CORS flag is unset.

        response
    };

    // Step 5.
    if response.as_ref().ok().map_or(false, |response| is_redirect_status(response.status)) {
        // Step 5.1.
        // Not applicable: the connection does not use HTTP/2.

        // Steps 5.2-4.
        // Not applicable: matters only if request's redirect mode is not "error".

        // Step 5.5.
        // Request's redirect mode is "error".
        response = Err(NetworkError::Internal("Response should not be a redirection.".into()));
    }

    // Step 6.
    response
}

// https://fetch.spec.whatwg.org/#concept-http-network-or-cache-fetch
fn http_network_or_cache_fetch(url: &ServoUrl,
                               origin: String,
                               headers: &mut Headers,
                               http_state: &HttpState)
                               -> Result<Response, NetworkError> {
    // Steps 1-3.
    // Not applicable: we don't even have a request yet, and there is no body
    // in a WebSocket request.

    // Step 4.
    // Not applicable: credentials flag is always set
    // because credentials mode is "include."

    // Steps 5-9.
    // Not applicable: there is no body in a WebSocket request.

    // Step 10.
    // TODO: handle header Referer.

    // Step 11.
    // Request's mode is "websocket".
    headers.set(Origin(origin));

    // Step 12.
    // TODO: handle header User-Agent.

    // Steps 13-14.
    // Not applicable: request's cache mode is "no-store".

    // Step 15.
    {
        // Step 15.1.
        // We know there is no Pragma header yet.
        headers.set(Pragma::NoCache);

        // Step 15.2.
        // We know there is no Cache-Control header yet.
        headers.set(CacheControl(vec![CacheDirective::NoCache]));
    }

    // Step 16.
    // TODO: handle Accept-Encoding.
    // Not applicable: Connection header is already present.
    // TODO: handle DNT.
    headers.set(Host {
        hostname: url.host_str().unwrap().to_owned(),
        port: url.port(),
    });

    // Step 17.
    // Credentials flag is set.
    {
        // Step 17.1.
        // TODO: handle user agent configured to block cookies.
        set_request_cookies(&url, headers, &http_state.cookie_jar);

        // Steps 17.2-6.
        // Not applicable: request has no Authorization header.
    }

    // Step 18.
    // TODO: proxy-authentication entry.

    // Step 19.
    // Not applicable: with step 21 being useless, this one is too.

    // Step 20.
    // Not applicable: revalidatingFlag is only useful if step 21 is.

    // Step 21.
    // Not applicable: cache mode is "no-store".

    // Step 22.
    // There is no response yet.
    let response = {
        // Step 22.1.
        // Not applicable: cache mode is "no-store".

        // Step 22.2.
        let forward_response = http_network_fetch(url, headers, http_state);

        // Step 22.3.
        // Not applicable: request's method is not unsafe.

        // Step 22.4.
        // Not applicable: revalidatingFlag is unset.

        // Step 22.5.
        // There is no response yet and the response should not be cached.
        forward_response
    };

    // Step 23.
    // TODO: handle 401 status when request's window is not "no-window".

    // Step 24.
    // TODO: handle 407 status when request's window is not "no-window".

    // Step 25.
    // Not applicable: authentication-fetch flag is unset.

    // Step 26.
    response
}

// https://fetch.spec.whatwg.org/#concept-http-network-fetch
fn http_network_fetch(url: &ServoUrl,
                      headers: &Headers,
                      http_state: &HttpState)
                      -> Result<Response, NetworkError> {
    // Step 1.
    // Not applicable: credentials flag is set.

    // Steps 2-3.
    // Request's mode is "websocket".
    let connection = obtain_a_websocket_connection(url)?;

    // Step 4.
    // Not applicable: request’s body is null.

    // Step 5.
    let response = make_request(connection, url, headers)?;

    // Steps 6-12.
    // Not applicable: correct WebSocket responses don't have a body.

    // Step 13.
    // TODO: handle response's CSP list.

    // Step 14.
    // Not applicable: request's cache mode is "no-store".

    // Step 15.
    if let Some(cookies) = response.headers.get::<SetCookie>() {
        let mut jar = http_state.cookie_jar.write().unwrap();
        for cookie in &**cookies {
            if let Some(cookie) = Cookie::from_cookie_string(cookie.clone(), url, CookieSource::HTTP) {
                jar.push(cookie, url, CookieSource::HTTP);
            }
        }
    }

    // Step 16.
    // Not applicable: correct WebSocket responses don't have a body.

    // Step 17.
    Ok(response)
}

fn make_request(mut stream: Stream,
                url: &ServoUrl,
                headers: &Headers)
                -> Result<Response, NetworkError> {
    write_request(&mut stream, url, headers).map_err(|e| {
        NetworkError::Internal(format!("Request could not be sent: {}", e))
    })?;

    // FIXME: Stream isn't supposed to be cloned.
    let writer = stream.clone();

    // FIXME: BufReader from hyper isn't supposed to be used.
    let mut reader = BufReader::new(stream);

    let head = parse_response(&mut reader).map_err(|e| {
        NetworkError::Internal(format!("Response could not be read: {}", e))
    })?;

    // This isn't in the spec, but this is the correct thing to do for WebSocket requests.
    if head.version != HttpVersion::Http11 {
        return Err(NetworkError::Internal("Response's HTTP version should be HTTP/1.1.".into()));
    }

    // FIXME: StatusCode::from_u16 isn't supposed to be used.
    let status = StatusCode::from_u16(head.subject.0);
    Ok(Response {
        status: status,
        headers: head.headers,
        reader: reader,
        writer: writer,
    })
}

fn write_request(stream: &mut Stream,
                 url: &ServoUrl,
                 headers: &Headers)
                 -> io::Result<()> {
    // Write "GET /foo/bar HTTP/1.1\r\n".
    let method = Method::Get;
    let request_uri = &url.as_url()[Position::BeforePath..Position::AfterQuery];
    let version = HttpVersion::Http11;
    write!(stream, "{} {} {}{}", method, request_uri, version, LINE_ENDING)?;

    // Write the headers.
    write!(stream, "{}{}", headers, LINE_ENDING)
}
