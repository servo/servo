/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie_storage::CookieStorage;
use http_loader;
use hyper::header::Host;
use net_traits::{WebSocketCommunicate, WebSocketConnectData, WebSocketDomAction, WebSocketNetworkEvent};
use net_traits::MessageData;
use net_traits::hosts::replace_hosts;
use net_traits::unwrap_websocket_protocol;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use util::thread::spawn_named;
use websocket::{Client, Message};
use websocket::header::{Headers, Origin, WebSocketProtocol};
use websocket::message::Type;
use websocket::receiver::Receiver;
use websocket::result::{WebSocketError, WebSocketResult};
use websocket::sender::Sender;
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WSReceiver;
use websocket::ws::sender::Sender as Sender_Object;
use websocket::ws::util::url::parse_url;

/// *Establish a WebSocket Connection* as defined in RFC 6455.
fn establish_a_websocket_connection(resource_url: &ServoUrl, net_url: (Host, String, bool),
                                    origin: String, protocols: Vec<String>,
                                    cookie_jar: Arc<RwLock<CookieStorage>>)
    -> WebSocketResult<(Headers, Sender<WebSocketStream>, Receiver<WebSocketStream>)> {
    let host = Host {
        hostname: resource_url.host_str().unwrap().to_owned(),
        port: resource_url.port_or_known_default(),
    };

    let mut request = try!(Client::connect(net_url));
    request.headers.set(Origin(origin));
    request.headers.set(host);
    if !protocols.is_empty() {
        request.headers.set(WebSocketProtocol(protocols.clone()));
    };

    http_loader::set_request_cookies(&resource_url, &mut request.headers, &cookie_jar);

    let response = try!(request.send());
    try!(response.validate());

    {
       let protocol_in_use = unwrap_websocket_protocol(response.protocol());
        if let Some(protocol_name) = protocol_in_use {
                if !protocols.is_empty() && !protocols.iter().any(|p| (&**p).eq_ignore_ascii_case(protocol_name)) {
                    return Err(WebSocketError::ProtocolError("Protocol in Use not in client-supplied protocol list"));
            };
        };
    }

    let headers = response.headers.clone();
    let (sender, receiver) = response.begin().split();
    Ok((headers, sender, receiver))

}

pub fn init(connect: WebSocketCommunicate, connect_data: WebSocketConnectData, cookie_jar: Arc<RwLock<CookieStorage>>) {
    spawn_named(format!("WebSocket connection to {}", connect_data.resource_url), move || {
        // Step 8: Protocols.

        // Step 9.

        // URL that we actually fetch from the network, after applying the replacements
        // specified in the hosts file.
        let net_url_result = parse_url(replace_hosts(&connect_data.resource_url).as_url().unwrap());
        let net_url = match net_url_result {
            Ok(net_url) => net_url,
            Err(e) => {
                debug!("Failed to establish a WebSocket connection: {:?}", e);
                let _ = connect.event_sender.send(WebSocketNetworkEvent::Fail);
                return;
            }
        };
        let channel = establish_a_websocket_connection(&connect_data.resource_url,
                                                       net_url,
                                                       connect_data.origin,
                                                       connect_data.protocols.clone(),
                                                       cookie_jar);
        let (_, ws_sender, mut receiver) = match channel {
            Ok(channel) => {
                let _ = connect.event_sender.send(WebSocketNetworkEvent::ConnectionEstablished(channel.0.clone(),
                                                                                               connect_data.protocols));
                channel
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

        let initiated_close_outgoing = initiated_close.clone();
        let ws_sender_outgoing = ws_sender.clone();
        let resource_action_receiver = connect.action_receiver;
        thread::spawn(move || {
            while let Ok(dom_action) = resource_action_receiver.recv() {
                match dom_action {
                    WebSocketDomAction::SendMessage(MessageData::Text(data)) => {
                        ws_sender_outgoing.lock().unwrap().send_message(&Message::text(data)).unwrap();
                    },
                    WebSocketDomAction::SendMessage(MessageData::Binary(data)) => {
                        ws_sender_outgoing.lock().unwrap().send_message(&Message::binary(data)).unwrap();
                    },
                    WebSocketDomAction::Close(code, reason) => {
                        if !initiated_close_outgoing.fetch_or(true, Ordering::SeqCst) {
                            let message = match code {
                                Some(code) => Message::close_because(code, reason.unwrap_or("".to_owned())),
                                None => Message::close()
                            };
                            ws_sender_outgoing.lock().unwrap().send_message(&message).unwrap();
                        }
                    },
                }
            }
        });
    });
}
