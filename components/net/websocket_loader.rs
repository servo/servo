/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie::Cookie;
use cookie_storage::CookieStorage;
use fetch::methods::should_be_blocked_due_to_bad_port;
use http_loader;
use hyper::header::{Host, SetCookie};
use net_traits::{CookieSource, MessageData, WebSocketCommunicate};
use net_traits::{WebSocketConnectData, WebSocketDomAction, WebSocketNetworkEvent};
use net_traits::hosts::replace_hosts;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use websocket::{Client, Message};
use websocket::header::{Origin, WebSocketProtocol};
use websocket::message::Type;
use websocket::receiver::Receiver;
use websocket::result::{WebSocketError, WebSocketResult};
use websocket::sender::Sender;
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WSReceiver;
use websocket::ws::sender::Sender as Sender_Object;

// https://fetch.spec.whatwg.org/#concept-websocket-establish
fn establish_a_websocket_connection(resource_url: &ServoUrl,
                                    origin: String,
                                    protocols: Vec<String>,
                                    cookie_jar: Arc<RwLock<CookieStorage>>)
                                    -> WebSocketResult<(Option<String>,
                                                        Sender<WebSocketStream>,
                                                        Receiver<WebSocketStream>)> {
    // Steps 1-2 are not really applicable here, given we don't exactly go
    // through the same infrastructure as the Fetch spec.

    if should_be_blocked_due_to_bad_port(resource_url) {
        // Subset of steps 11-12, we inline the bad port check here from the
        // main fetch algorithm for the same reason steps 1-2 are not
        // applicable.
        return Err(WebSocketError::RequestError("Request should be blocked due to bad port."));
    }

    // Steps 3-7.
    let net_url = replace_hosts(resource_url);
    let mut request = try!(Client::connect(net_url.as_url()));

    // Client::connect sets the Host header to the host of the URL that is
    // passed to it, so we need to reset it afterwards to the correct one.
    request.headers.set(Host {
        hostname: resource_url.host_str().unwrap().to_owned(),
        port: resource_url.port(),
    });

    // Step 8.
    if !protocols.is_empty() {
        request.headers.set(WebSocketProtocol(protocols.clone()));
    }

    // Steps 9-10.
    // TODO: support for permessage-deflate extension.

    // Subset of step 11.
    // See step 2 of https://fetch.spec.whatwg.org/#concept-fetch.
    request.headers.set(Origin(origin));

    // Transitive subset of step 11.
    // See step 17.1 of https://fetch.spec.whatwg.org/#concept-http-network-or-cache-fetch.
    http_loader::set_request_cookies(&resource_url, &mut request.headers, &cookie_jar);

    // Step 11, somewhat.
    let response = try!(request.send());

    // Step 12, 14.
    try!(response.validate());

    // Step 13 and transitive subset of step 14.
    // See step 6 of http://tools.ietf.org/html/rfc6455#section-4.1.
    let protocol_in_use = response.protocol().and_then(|header| {
        // https://github.com/whatwg/fetch/issues/515
        header.first().cloned()
    });
    if let Some(ref protocol_name) = protocol_in_use {
        if !protocols.is_empty() && !protocols.iter().any(|p| (&**p).eq_ignore_ascii_case(protocol_name)) {
            return Err(WebSocketError::ProtocolError("Protocol in Use not in client-supplied protocol list"));
        };
    };

    // Transitive subset of step 11.
    // See step 15 of https://fetch.spec.whatwg.org/#http-network-fetch.
    if let Some(cookies) = response.headers.get::<SetCookie>() {
        let mut jar = cookie_jar.write().unwrap();
        for cookie in &**cookies {
            if let Some(cookie) = Cookie::new_wrapped(cookie.clone(), resource_url, CookieSource::HTTP) {
                jar.push(cookie, resource_url, CookieSource::HTTP);
            }
        }
    }

    let (sender, receiver) = response.begin().split();
    Ok((protocol_in_use, sender, receiver))
}

pub fn init(connect: WebSocketCommunicate, connect_data: WebSocketConnectData, cookie_jar: Arc<RwLock<CookieStorage>>) {
    thread::Builder::new().name(format!("WebSocket connection to {}", connect_data.resource_url)).spawn(move || {
        let channel = establish_a_websocket_connection(&connect_data.resource_url,
                                                       connect_data.origin,
                                                       connect_data.protocols,
                                                       cookie_jar);
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
                    Type::Text => MessageData::Text(String::from_utf8_lossy(&message.payload).into_owned()),
                    Type::Binary => MessageData::Binary(message.payload.into_owned()),
                    Type::Ping => {
                        let pong = Message::pong(message.payload);
                        ws_sender_incoming.lock().unwrap().send_message(&pong).unwrap();
                        continue;
                    },
                    Type::Pong => continue,
                    Type::Close => {
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
