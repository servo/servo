/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets](https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#json-packets).

use std::io::{Read, Write};
use std::net::TcpStream;

use log::debug;
use serde::Serialize;
use serde_json::{self, Value, json};

use crate::actor::ActorError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDescription {
    pub category: &'static str,
    pub type_name: &'static str,
    pub methods: Vec<Method>,
}

#[derive(Serialize)]
pub struct Method {
    pub name: &'static str,
    pub request: Value,
    pub response: Value,
}

pub trait JsonPacketStream {
    fn write_json_packet<T: Serialize>(&mut self, message: &T) -> Result<(), ActorError>;
    fn read_json_packet(&mut self) -> Result<Option<Value>, String>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<T: Serialize>(&mut self, message: &T) -> Result<(), ActorError> {
        let s = serde_json::to_string(message).map_err(|_| ActorError::Internal)?;
        debug!("<- {}", s);
        write!(self, "{}:{}", s.len(), s).map_err(|_| ActorError::Internal)?;
        Ok(())
    }

    fn read_json_packet(&mut self) -> Result<Option<Value>, String> {
        // https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#stream-transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec![];
        loop {
            let mut buf = [0];
            let byte = match self.read(&mut buf) {
                Ok(0) => return Ok(None), // EOF
                Ok(1) => buf[0],
                Ok(_) => unreachable!(),
                Err(e) => return Err(e.to_string()),
            };
            match byte {
                b':' => {
                    let packet_len_str = match String::from_utf8(buffer) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("nonvalid UTF8 in packet length".to_owned()),
                    };
                    let packet_len = match packet_len_str.parse::<u64>() {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("packet length missing / not parsable".to_owned()),
                    };
                    let mut packet = String::new();
                    self.take(packet_len)
                        .read_to_string(&mut packet)
                        .map_err(|e| e.to_string())?;
                    debug!("{}", packet);
                    return match serde_json::from_str(&packet) {
                        Ok(json) => Ok(Some(json)),
                        Err(err) => Err(err.to_string()),
                    };
                },
                c => buffer.push(c),
            }
        }
    }
}

/// Wrapper around a client stream that guarantees request/reply invariants.
///
/// Client messages, which are always requests, are dispatched to Actor instances one at a time via
/// [`crate::Actor::handle_message`]. Each request must be paired with exactly one reply from the
/// same actor the request was sent to, where a reply is a message with no type (if a message from
/// the server has a type, it’s a notification, not a reply).
///
/// Failing to reply to a request will almost always permanently break that actor, because either
/// the client gets stuck waiting for a reply, or the client receives the reply for a subsequent
/// request as if it was the reply for the current request. If an actor fails to reply to a request,
/// we want the dispatcher ([`crate::ActorRegistry::handle_message`]) to send an error of type
/// `unrecognizedPacketType`, to keep the conversation for that actor in sync.
///
/// Since replies come in all shapes and sizes, we want to allow Actor types to send replies without
/// having to return them to the dispatcher. This wrapper type allows the dispatcher to check if a
/// valid reply was sent, and guarantees that if the actor tries to send a reply, it’s actually a
/// valid reply (see [`Self::is_valid_reply`]).
///
/// It does not currently guarantee anything about messages sent via the [`TcpStream`] released via
/// [`Self::try_clone_stream`] or the return value of [`Self::reply`].
pub struct ClientRequest<'req, 'sent> {
    /// Client stream.
    stream: &'req mut TcpStream,
    /// Expected actor name.
    actor_name: &'req str,
    /// Sent flag, allowing ActorRegistry to check for unhandled requests.
    sent: &'sent mut bool,
}

impl ClientRequest<'_, '_> {
    /// Run the given handler, with a new request that wraps the given client stream and expected actor name.
    ///
    /// Returns [`ActorError::UnrecognizedPacketType`] if the actor did not send a reply.
    pub fn handle<'req>(
        client: &'req mut TcpStream,
        actor_name: &'req str,
        handler: impl FnOnce(ClientRequest<'req, '_>) -> Result<(), ActorError>,
    ) -> Result<(), ActorError> {
        let mut sent = false;
        let request = ClientRequest {
            stream: client,
            actor_name,
            sent: &mut sent,
        };
        handler(request)?;

        if sent {
            Ok(())
        } else {
            Err(ActorError::UnrecognizedPacketType)
        }
    }
}

impl<'req> ClientRequest<'req, '_> {
    /// Send the given reply to the request being handled.
    ///
    /// If successful, sets the sent flag and returns the underlying stream,
    /// allowing other messages to be sent after replying to a request.
    pub fn reply<T: Serialize>(self, reply: &T) -> Result<&'req mut TcpStream, ActorError> {
        debug_assert!(self.is_valid_reply(reply), "Message is not a valid reply");
        self.stream.write_json_packet(reply)?;
        *self.sent = true;
        Ok(self.stream)
    }

    /// Like `reply`, but for cases where the actor no longer needs the stream.
    pub fn reply_final<T: Serialize>(self, reply: &T) -> Result<(), ActorError> {
        debug_assert!(self.is_valid_reply(reply), "Message is not a valid reply");
        let _stream = self.reply(reply)?;
        Ok(())
    }

    pub fn try_clone_stream(&self) -> std::io::Result<TcpStream> {
        self.stream.try_clone()
    }

    /// Return true iff the given message is a reply (has no `type` or `to`), and is from the expected actor.
    ///
    /// This incurs a runtime conversion to a BTreeMap, so it should only be used in debug assertions.
    fn is_valid_reply<T: Serialize>(&self, message: &T) -> bool {
        let reply = json!(message);
        reply.get("from").and_then(|from| from.as_str()) == Some(self.actor_name) &&
            reply.get("to").is_none() &&
            reply.get("type").is_none()
    }
}

/// Actors can also send other messages before replying to a request.
impl JsonPacketStream for ClientRequest<'_, '_> {
    fn write_json_packet<T: Serialize>(&mut self, message: &T) -> Result<(), ActorError> {
        debug_assert!(
            !self.is_valid_reply(message),
            "Replies must use reply() or reply_final()"
        );
        self.stream.write_json_packet(message)
    }

    fn read_json_packet(&mut self) -> Result<Option<Value>, String> {
        self.stream.read_json_packet()
    }
}
