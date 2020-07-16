/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The websocket handler has three main responsibilities:
//! 1) initiate the initial HTTP connection and process the response
//! 2) ensure any DOM requests for sending/closing are propagated to the network
//! 3) transmit any incoming messages/closing to the DOM
//!
//! In order to accomplish this, the handler uses a long-running loop that selects
//! over events from the network and events from the DOM, using async/await to avoid
//! the need for a dedicated thread per websocket.

use crate::connector::{create_tls_config, ALPN_H1};
use crate::cookie::Cookie;
use crate::fetch::methods::should_be_blocked_due_to_bad_port;
use crate::hosts::replace_host;
use crate::http_loader::HttpState;
use async_tungstenite::tokio::{client_async_tls_with_connector_and_config, ConnectStream};
use async_tungstenite::WebSocketStream;
use embedder_traits::resources::{self, Resource};
use futures03::future::TryFutureExt;
use futures03::sink::SinkExt;
use futures03::stream::StreamExt;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::request::{RequestBuilder, RequestMode};
use net_traits::{CookieSource, MessageData};
use net_traits::{WebSocketDomAction, WebSocketNetworkEvent};
use openssl::ssl::ConnectConfiguration;
use servo_url::ServoUrl;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio2::net::TcpStream;
use tokio2::runtime::Runtime;
use tokio2::select;
use tokio2::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tungstenite::error::Error;
use tungstenite::error::Result as WebSocketResult;
use tungstenite::handshake::client::{Request, Response};
use tungstenite::http::header::{self as WSHeader, HeaderValue as WSHeaderValue};
use tungstenite::protocol::CloseFrame;
use tungstenite::Message;
use url::Url;

// Websockets get their own tokio runtime that's independent of the one used for
// HTTP connections, otherwise a large number of websockets could occupy all workers
// and starve other network traffic.
lazy_static! {
    pub static ref HANDLE: Mutex<Option<Runtime>> = Mutex::new(Some(Runtime::new().unwrap()));
}

/// Create a tungstenite Request object for the initial HTTP request.
/// This request contains `Origin`, `Sec-WebSocket-Protocol`, `Authorization`,
/// and `Cookie` headers as appropriate.
/// Returns an error if any header values are invalid or tungstenite cannot create
/// the desired request.
fn create_request(
    resource_url: &ServoUrl,
    origin: &str,
    protocols: &[String],
    http_state: &HttpState,
) -> WebSocketResult<Request> {
    let mut builder = Request::get(resource_url.as_str());
    let headers = builder.headers_mut().unwrap();
    headers.insert("Origin", WSHeaderValue::from_str(origin)?);

    if !protocols.is_empty() {
        let protocols = protocols.join(",");
        headers.insert(
            "Sec-WebSocket-Protocol",
            WSHeaderValue::from_str(&protocols)?,
        );
    }

    let mut cookie_jar = http_state.cookie_jar.write().unwrap();
    cookie_jar.remove_expired_cookies_for_url(resource_url);
    if let Some(cookie_list) = cookie_jar.cookies_for_url(resource_url, CookieSource::HTTP) {
        headers.insert("Cookie", WSHeaderValue::from_str(&cookie_list)?);
    }

    if resource_url.password().is_some() || resource_url.username() != "" {
        let basic = base64::encode(&format!(
            "{}:{}",
            resource_url.username(),
            resource_url.password().unwrap_or("")
        ));
        headers.insert(
            "Authorization",
            WSHeaderValue::from_str(&format!("Basic {}", basic))?,
        );
    }

    let request = builder.body(())?;
    Ok(request)
}

/// Process an HTTP response resulting from a WS handshake.
/// This ensures that any `Cookie` or HSTS headers are recognized.
/// Returns an error if the protocol selected by the handshake doesn't
/// match the list of provided protocols in the original request.
fn process_ws_response(
    http_state: &HttpState,
    response: &Response,
    resource_url: &ServoUrl,
    protocols: &[String],
) -> Result<Option<String>, Error> {
    trace!("processing websocket http response for {}", resource_url);
    let mut protocol_in_use = None;
    if let Some(protocol_name) = response.headers().get("Sec-WebSocket-Protocol") {
        let protocol_name = protocol_name.to_str().unwrap();
        if !protocols.is_empty() && !protocols.iter().any(|p| protocol_name == (*p)) {
            return Err(Error::Protocol(
                "Protocol in use not in client-supplied protocol list".into(),
            ));
        }
        protocol_in_use = Some(protocol_name.to_string());
    }

    let mut jar = http_state.cookie_jar.write().unwrap();
    // TODO(eijebong): Replace thise once typed headers settled on a cookie impl
    for cookie in response.headers().get_all(WSHeader::SET_COOKIE) {
        if let Ok(s) = std::str::from_utf8(cookie.as_bytes()) {
            if let Some(cookie) =
                Cookie::from_cookie_string(s.into(), resource_url, CookieSource::HTTP)
            {
                jar.push(cookie, resource_url, CookieSource::HTTP);
            }
        }
    }

    // We need to make a new header map here because tungstenite depends on
    // a more recent version of http than the rest of the network stack, so the
    // HeaderMap types are incompatible.
    let mut headers = HeaderMap::new();
    for (key, value) in response.headers().iter() {
        if let (Ok(key), Ok(value)) = (
            HeaderName::from_bytes(key.as_ref()),
            HeaderValue::from_bytes(value.as_ref()),
        ) {
            headers.insert(key, value);
        }
    }
    http_state
        .hsts_list
        .write()
        .unwrap()
        .update_hsts_list_from_response(resource_url, &headers);

    Ok(protocol_in_use)
}

#[derive(Debug)]
enum DomMsg {
    Send(Message),
    Close(Option<(u16, String)>),
}

/// Initialize a listener for DOM actions. These are routed from the IPC channel
/// to a tokio channel that the main WS client task uses to receive them.
fn setup_dom_listener(
    dom_action_receiver: IpcReceiver<WebSocketDomAction>,
    initiated_close: Arc<AtomicBool>,
) -> UnboundedReceiver<DomMsg> {
    let (sender, receiver) = unbounded_channel();

    ROUTER.add_route(
        dom_action_receiver.to_opaque(),
        Box::new(move |message| {
            let dom_action = message.to().expect("Ws dom_action message to deserialize");
            trace!("handling WS DOM action: {:?}", dom_action);
            match dom_action {
                WebSocketDomAction::SendMessage(MessageData::Text(data)) => {
                    if let Err(e) = sender.send(DomMsg::Send(Message::Text(data))) {
                        warn!("Error sending websocket message: {:?}", e);
                    }
                },
                WebSocketDomAction::SendMessage(MessageData::Binary(data)) => {
                    if let Err(e) = sender.send(DomMsg::Send(Message::Binary(data))) {
                        warn!("Error sending websocket message: {:?}", e);
                    }
                },
                WebSocketDomAction::Close(code, reason) => {
                    if initiated_close.fetch_or(true, Ordering::SeqCst) {
                        return;
                    }
                    let frame = code.map(move |c| (c, reason.unwrap_or_default()));
                    if let Err(e) = sender.send(DomMsg::Close(frame)) {
                        warn!("Error closing websocket: {:?}", e);
                    }
                },
            }
        }),
    );

    receiver
}

/// Listen for WS events from the DOM and the network until one side
/// closes the connection or an error occurs. Since this is an async
/// function that uses the select operation, it will run as a task
/// on the WS tokio runtime.
async fn run_ws_loop(
    mut dom_receiver: UnboundedReceiver<DomMsg>,
    resource_event_sender: IpcSender<WebSocketNetworkEvent>,
    mut stream: WebSocketStream<ConnectStream>,
) {
    loop {
        select! {
            dom_msg = dom_receiver.recv() => {
                trace!("processing dom msg: {:?}", dom_msg);
                let dom_msg = match dom_msg {
                    Some(msg) => msg,
                    None => break,
                };
                match dom_msg {
                    DomMsg::Send(m) => {
                        if let Err(e) = stream.send(m).await {
                            warn!("error sending websocket message: {:?}", e);
                        }
                    },
                    DomMsg::Close(frame) => {
                        if let Err(e) = stream.close(frame.map(|(code, reason)| {
                            CloseFrame {
                                code: code.into(),
                                reason: reason.into(),
                            }
                        })).await {
                            warn!("error closing websocket: {:?}", e);
                        }
                    },
                }
            }
            ws_msg = stream.next() => {
                trace!("processing WS stream: {:?}", ws_msg);
                let msg = match ws_msg {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        warn!("Error in WebSocket communication: {:?}", e);
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Fail);
                        break;
                    },
                    None => {
                        warn!("Error in WebSocket communication");
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Fail);
                        break;
                    }
                };
                match msg {
                    Message::Text(s) => {
                        let message = MessageData::Text(s);
                        if let Err(e) = resource_event_sender
                            .send(WebSocketNetworkEvent::MessageReceived(message))
                        {
                            warn!("Error sending websocket notification: {:?}", e);
                            break;
                        }
                    }

                    Message::Binary(v) => {
                        let message = MessageData::Binary(v);
                        if let Err(e) = resource_event_sender
                            .send(WebSocketNetworkEvent::MessageReceived(message))
                        {
                            warn!("Error sending websocket notification: {:?}", e);
                            break;
                        }
                    }

                    Message::Ping(_) | Message::Pong(_) => {}

                    Message::Close(frame) => {
                        let (reason, code) = match frame {
                            Some(frame) => (frame.reason, Some(frame.code.into())),
                            None => ("".into(), None),
                        };
                        debug!("Websocket connection closing due to ({:?}) {}", code, reason);
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Close(
                            code,
                            reason.to_string(),
                        ));
                        break;
                    }
                }
            }
        }
    }
}

/// Initiate a new async WS connection. Returns an error if the connection fails
/// for any reason, or if the response isn't valid. Otherwise, the endless WS
/// listening loop will be started.
async fn start_websocket(
    http_state: Arc<HttpState>,
    url: ServoUrl,
    resource_event_sender: IpcSender<WebSocketNetworkEvent>,
    protocols: Vec<String>,
    client: Request,
    tls_config: ConnectConfiguration,
    dom_action_receiver: IpcReceiver<WebSocketDomAction>,
) -> Result<(), Error> {
    trace!("starting WS connection to {}", url);

    let initiated_close = Arc::new(AtomicBool::new(false));
    let dom_receiver = setup_dom_listener(dom_action_receiver, initiated_close.clone());

    let host_str = client
        .uri()
        .host()
        .ok_or_else(|| Error::Url("No host string".into()))?;
    let host = replace_host(host_str);
    let mut net_url =
        Url::parse(&client.uri().to_string()).map_err(|e| Error::Url(e.to_string().into()))?;
    net_url
        .set_host(Some(&host))
        .map_err(|e| Error::Url(e.to_string().into()))?;

    let domain = net_url
        .host()
        .ok_or_else(|| Error::Url("No host string".into()))?;
    let port = net_url
        .port_or_known_default()
        .ok_or_else(|| Error::Url("Unknown port".into()))?;

    let try_socket = TcpStream::connect((&*domain.to_string(), port)).await;
    let socket = try_socket.map_err(Error::Io)?;
    let (stream, response) =
        client_async_tls_with_connector_and_config(client, socket, Some(tls_config), None).await?;

    let protocol_in_use = process_ws_response(&http_state, &response, &url, &protocols)?;

    if !initiated_close.load(Ordering::SeqCst) {
        if resource_event_sender
            .send(WebSocketNetworkEvent::ConnectionEstablished { protocol_in_use })
            .is_err()
        {
            return Ok(());
        }

        trace!("about to start ws loop for {}", url);
        run_ws_loop(dom_receiver, resource_event_sender, stream).await;
    } else {
        trace!("client closed connection for {}, not running loop", url);
    }
    Ok(())
}

/// Create a new websocket connection for the given request.
fn connect(
    mut req_builder: RequestBuilder,
    resource_event_sender: IpcSender<WebSocketNetworkEvent>,
    dom_action_receiver: IpcReceiver<WebSocketDomAction>,
    http_state: Arc<HttpState>,
    certificate_path: Option<String>,
) -> Result<(), String> {
    let protocols = match req_builder.mode {
        RequestMode::WebSocket { protocols } => protocols,
        _ => {
            return Err(
                "Received a RequestBuilder with a non-websocket mode in websocket_loader"
                    .to_string(),
            )
        },
    };

    // https://fetch.spec.whatwg.org/#websocket-opening-handshake
    // By standard, we should work with an http(s):// URL (req_url),
    // but as ws-rs expects to be called with a ws(s):// URL (net_url)
    // we upgrade ws to wss, so we don't have to convert http(s) back to ws(s).
    http_state
        .hsts_list
        .read()
        .unwrap()
        .apply_hsts_rules(&mut req_builder.url);

    let scheme = req_builder.url.scheme();
    let mut req_url = req_builder.url.clone();
    match scheme {
        "ws" => {
            req_url
                .as_mut_url()
                .set_scheme("http")
                .map_err(|()| "couldn't replace scheme".to_string())?;
        },
        "wss" => {
            req_url
                .as_mut_url()
                .set_scheme("https")
                .map_err(|()| "couldn't replace scheme".to_string())?;
        },
        _ => {},
    }

    if should_be_blocked_due_to_bad_port(&req_url) {
        return Err("Port blocked".to_string());
    }

    let certs = match certificate_path {
        Some(ref path) => fs::read_to_string(path).map_err(|e| e.to_string())?,
        None => resources::read_string(Resource::SSLCertificates),
    };

    let client = match create_request(
        &req_builder.url,
        &req_builder.origin.ascii_serialization(),
        &protocols,
        &*http_state,
    ) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };

    let tls_config = create_tls_config(
        &certs,
        ALPN_H1,
        http_state.extra_certs.clone(),
        http_state.connection_certs.clone(),
    );
    let tls_config = match tls_config.build().configure() {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };

    let resource_event_sender2 = resource_event_sender.clone();
    match HANDLE.lock().unwrap().as_mut() {
        Some(handle) => handle.spawn(
            start_websocket(
                http_state,
                req_builder.url.clone(),
                resource_event_sender,
                protocols,
                client,
                tls_config,
                dom_action_receiver,
            )
            .map_err(move |e| {
                warn!("Failed to establish a WebSocket connection: {:?}", e);
                let _ = resource_event_sender2.send(WebSocketNetworkEvent::Fail);
            }),
        ),
        None => return Err("No runtime available".to_string()),
    };
    Ok(())
}

/// Create a new websocket connection for the given request.
pub fn init(
    req_builder: RequestBuilder,
    resource_event_sender: IpcSender<WebSocketNetworkEvent>,
    dom_action_receiver: IpcReceiver<WebSocketDomAction>,
    http_state: Arc<HttpState>,
    certificate_path: Option<String>,
) {
    let resource_event_sender2 = resource_event_sender.clone();
    if let Err(e) = connect(
        req_builder,
        resource_event_sender,
        dom_action_receiver,
        http_state,
        certificate_path,
    ) {
        warn!("Error starting websocket: {}", e);
        let _ = resource_event_sender2.send(WebSocketNetworkEvent::Fail);
    }
}
