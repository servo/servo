/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cookie_storage::CookieStorage;
use http_loader::set_request_cookies;
use hyper::header::Host;
use ipc_channel::ipc::IpcSender;
use net_traits::{WebSocketCommunicate, WebSocketConnectData, WebSocketDomAction};
use net_traits::{WebSocketHeaders, WebSocketNetworkEvent};
use net_traits::MessageData;
use net_traits::hosts::replace_hosts;
use net_traits::parse_url;
use servo_url::ServoUrl;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::thread;
use url::Url;
use ws::{CloseCode, Factory, Handler, Handshake, Message, Request, Response, Sender, WebSocket};
use ws::{Error as WebSocketError, ErrorKind as WebSocketErrorKind, Result as WebSocketResult};

/// A client for connecting to a websocket server
#[derive(Clone, Copy)]
struct Client<'a> {
    origin: &'a str,
    host: &'a Host,
    protocols: &'a [String],
    cookie_jar: &'a Arc<RwLock<CookieStorage>>,
    resource_url: &'a ServoUrl,
    event_sender: &'a IpcSender<WebSocketNetworkEvent>,
}

impl<'a> Factory for Client<'a> {
    type Handler = Self;

    fn connection_made(&mut self, _: Sender) -> Self::Handler {
        *self
    }

    fn connection_lost(&mut self, _: Self::Handler) {
        let _ = self.event_sender.send(WebSocketNetworkEvent::Fail);
    }
}

impl<'a> Handler for Client<'a> {
    fn build_request(&mut self, url: &Url) -> WebSocketResult<Request> {
        debug!("Handler is building request from {}.", url);
        try!(parse_url(url).map_err(|e| WebSocketError::new(WebSocketErrorKind::Protocol, e.description().to_owned())));
        let mut req = try!(Request::from_url(url));
        req.headers_mut().push(("Origin".to_string(), self.origin.as_bytes().to_owned()));
        req.headers_mut().push(("Host".to_string(), format!("{}", self.host).as_bytes().to_owned()));
        for protocol in self.protocols {
            req.add_protocol(protocol);
        };
        let mut headers = WebSocketHeaders::new(req.headers().clone()).as_hyper_headers();
        set_request_cookies(self.resource_url, &mut headers, self.cookie_jar);
        if let Some(cookie) = headers.get_raw("Cookie") {
            req.headers_mut().push(("Cookie".to_string(), cookie[0].to_owned()));
        }
        Ok(req)
    }

    fn on_open(&mut self, shake: Handshake) -> WebSocketResult<()> {
        let headers = WebSocketHeaders::new(shake.response.headers().clone());
        let _ = self.event_sender.send(
            WebSocketNetworkEvent::ConnectionEstablished(headers, self.protocols.to_owned()));
        Ok(())
    }

    fn on_message(&mut self, message: Message) -> WebSocketResult<()> {
        let message = match message {
            Message::Text(message) => MessageData::Text(message),
            Message::Binary(message) => MessageData::Binary(message),
        };
        let _ = self.event_sender.send(WebSocketNetworkEvent::MessageReceived(message));
        Ok(())
    }

    fn on_error(&mut self, err: WebSocketError) {
        debug!("Error in WebSocket communication: {:?}", err);
        let _ = self.event_sender.send(WebSocketNetworkEvent::Fail);
    }

    fn on_response(&mut self, res: &Response) -> WebSocketResult<()> {
        let protocol_in_use = try!(res.protocol());
        if let Some(protocol_name) = protocol_in_use {
            let protocol_name = protocol_name.to_lowercase();
            if !self.protocols.is_empty() && !self.protocols.iter().any(|p| protocol_name == (*p).to_lowercase()) {
                let error = WebSocketError::new(WebSocketErrorKind::Protocol,
                                                "Protocol in Use not in client-supplied protocol list");
                return Err(error);
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        debug!("Connection closing due to ({:?}) {}", code, reason);
        let _ = self.event_sender.send(WebSocketNetworkEvent::Close(Some(code.into()), reason.to_owned()));
    }
}

pub fn init(connect: WebSocketCommunicate, connect_data: WebSocketConnectData, cookie_jar: Arc<RwLock<CookieStorage>>) {
    thread::Builder::new().name(format!("WebSocket connection to {}", connect_data.resource_url)).spawn(move || {
        // Step 8: Protocols.

        // Step 9.

        // URL that we actually fetch from the network, after applying the replacements
        // specified in the hosts file.
        let net_url = replace_hosts(&connect_data.resource_url).into_url().unwrap();

        let host = Host {
            hostname: connect_data.resource_url.host_str().unwrap().to_owned(),
            port: connect_data.resource_url.port_or_known_default(),
        };

        let protocols = connect_data.protocols.iter().map(|x| x.to_lowercase()).collect::<Vec<String>>();

        let client = Client {
            origin: &connect_data.origin,
            host: &host,
            protocols: &protocols,
            cookie_jar: &cookie_jar,
            resource_url: &connect_data.resource_url,
            event_sender: &connect.event_sender,
        };
        let mut ws = WebSocket::new(client).unwrap();
        if let Err(e) = ws.connect(net_url) {
            debug!("Failed to establish a WebSocket connection: {:?}", e);
            let _ = connect.event_sender.send(WebSocketNetworkEvent::Fail);
            return;
        };
        let sender = ws.broadcaster();
        let action_receiver = connect.action_receiver;
        thread::spawn(move || {
            while let Ok(dom_action) = action_receiver.recv() {
                let _ = match dom_action {
                    WebSocketDomAction::SendMessage(MessageData::Text(data)) => {
                        sender.send(Message::text(data))
                    },
                    WebSocketDomAction::SendMessage(MessageData::Binary(data)) => {
                        sender.send(Message::binary(data))
                    },
                    WebSocketDomAction::Close(Some(code), reason) => {
                        sender.close_with_reason(CloseCode::from(code), reason.unwrap_or("".to_owned()))
                    },
                    WebSocketDomAction::Close(None, reason) => {
                        sender.close_with_reason(CloseCode::Status, reason.unwrap_or("".to_owned()))
                    },
                };
            }
        });
        if let Err(e) = ws.run() {
            debug!("Failed to run WebSocket: {:?}", e);
            let _ = connect.event_sender.send(WebSocketNetworkEvent::Fail);
        };
    }).expect("Thread spawning failed");
}
