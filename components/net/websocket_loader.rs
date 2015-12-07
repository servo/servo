/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use hyper::header::Host;
use net_traits::MessageData;
use net_traits::hosts::replace_hosts;
use net_traits::{WebSocketCommunicate, WebSocketConnectData, WebSocketDomAction, WebSocketNetworkEvent};
use std::sync::{Arc, Mutex};
use std::thread;
use util::task::spawn_named;
use websocket::client::receiver::Receiver;
use websocket::client::request::Url;
use websocket::client::sender::Sender;
use websocket::header::Origin;
use websocket::message::Type;
use websocket::result::WebSocketResult;
use websocket::stream::WebSocketStream;
use websocket::ws::receiver::Receiver as WSReceiver;
use websocket::ws::sender::Sender as Sender_Object;
use websocket::ws::util::url::parse_url;
use websocket::{Client, Message};

/// *Establish a WebSocket Connection* as defined in RFC 6455.
fn establish_a_websocket_connection(resource_url: &Url, net_url: (Host, String, bool),
                                    origin: String)
    -> WebSocketResult<(Sender<WebSocketStream>, Receiver<WebSocketStream>)> {

    let host = Host {
        hostname: resource_url.serialize_host().unwrap(),
        port: resource_url.port_or_default()
    };

    let mut request = try!(Client::connect(net_url));
    request.headers.set(Origin(origin));
    request.headers.set(host);

    let response = try!(request.send());
    try!(response.validate());

    Ok(response.begin().split())
}

pub fn init(connect: WebSocketCommunicate, connect_data: WebSocketConnectData) {
    spawn_named(format!("WebSocket connection to {}", connect_data.resource_url), move || {
        // Step 8: Protocols.

        // Step 9.

        // URL that we actually fetch from the network, after applying the replacements
        // specified in the hosts file.
        let net_url_result = parse_url(&replace_hosts(&connect_data.resource_url));
        let net_url = match net_url_result {
            Ok(net_url) => net_url,
            Err(e) => {
                debug!("Failed to establish a WebSocket connection: {:?}", e);
                let _ = connect.event_sender.send(WebSocketNetworkEvent::Close);
                return;
            }
        };
        let channel = establish_a_websocket_connection(&connect_data.resource_url,
                                                       net_url,
                                                       connect_data.origin);
        let (ws_sender, mut receiver) = match channel {
            Ok(channel) => {
                let _ = connect.event_sender.send(WebSocketNetworkEvent::ConnectionEstablished);
                channel
            },
            Err(e) => {
                debug!("Failed to establish a WebSocket connection: {:?}", e);
                let _ = connect.event_sender.send(WebSocketNetworkEvent::Fail);
                return;
            }

        };

        let ws_sender = Arc::new(Mutex::new(ws_sender));

        let ws_sender_incoming = ws_sender.clone();
        let resource_event_sender = connect.event_sender;
        thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message: Message = match message {
                    Ok(m) => m,
                    Err(e) => {
                        debug!("Error receiving incoming WebSocket message: {:?}", e);
                        resource_event_sender.send(WebSocketNetworkEvent::Fail);
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
                        ws_sender_incoming.lock().unwrap().send_message(&message).unwrap();
                        let _ = resource_event_sender.send(WebSocketNetworkEvent::Close);
                        break;
                    },
                };
                let _ = resource_event_sender.send(WebSocketNetworkEvent::MessageReceived(message));
            }
        });

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
                        ws_sender_outgoing.lock().unwrap()
                        .send_message(&Message::close_because(code, reason)).unwrap();
                    },
                }
            }
        });
    });
}
