/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets](https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#json-packets).

use std::io::{self, ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};

use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Value, json};

use crate::actor::ActorError;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActorDescription {
    pub category: &'static str,
    pub type_name: &'static str,
    pub methods: Vec<Method>,
}

#[derive(Serialize)]
pub(crate) struct Method {
    pub name: &'static str,
    pub request: Value,
    pub response: Value,
}

pub trait JsonPacketStream {
    fn write_json_packet<T: Serialize>(&mut self, message: &T) -> Result<(), ActorError>;
    fn read_json_packet(&mut self) -> Result<Option<Value>, String>;
}

/// Wraps a Remote Debugging Protocol TCP stream, guaranteeing that network
/// operations are synchronized when cloning across threads.
#[derive(Clone, MallocSizeOf)]
pub(crate) struct DevtoolsConnection {
    /// Copy of [`TcpStream`] handle to use for receiving from the client.
    ///
    /// `TcpStream::read` is a mutating I/O operation that doesn't fit with `RwLock`.
    /// We clone a single stream handle into two mutexes so we can still lock
    /// reads and writes independently.
    #[conditional_malloc_size_of]
    receiver: Arc<Mutex<TcpStream>>,
    /// Copy of [`TcpStream`] handle to use for sending bytes to the client.
    #[conditional_malloc_size_of]
    sender: Arc<Mutex<TcpStream>>,
}

impl From<TcpStream> for DevtoolsConnection {
    fn from(value: TcpStream) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(value.try_clone().unwrap())),
            sender: Arc::new(Mutex::new(value)),
        }
    }
}

impl DevtoolsConnection {
    /// Calls [`TcpStream::peer_addr`] on the underlying stream.
    pub(crate) fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.receiver.lock().unwrap().peer_addr()
    }

    /// Calls [`TcpStream::shutdown`] on the underlying stream.
    pub(crate) fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.receiver.lock().unwrap().shutdown(how)
    }
}

impl JsonPacketStream for DevtoolsConnection {
    fn write_json_packet<T: serde::Serialize>(&mut self, message: &T) -> Result<(), ActorError> {
        let s = serde_json::to_string(message).map_err(|_| ActorError::Internal)?;
        log::debug!("<- {}", s);
        let mut stream = self.sender.lock().unwrap();
        write!(*stream, "{}:{}", s.len(), s).map_err(|_| ActorError::Internal)
    }

    fn read_json_packet(&mut self) -> Result<Option<Value>, String> {
        // https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#stream-transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec![];
        // Guard should be held until exactly one complete message has been read.
        let mut stream = self.receiver.lock().unwrap();
        loop {
            let mut buf = [0];
            match (*stream).read(&mut buf) {
                Ok(0) => return Ok(None), // EOF
                Ok(1) if buf[0] == b':' => {
                    let packet_len_str = String::from_utf8(buffer)
                        .map_err(|_| "nonvalid UTF8 in packet length".to_owned())?;
                    let packet_len = packet_len_str
                        .parse::<u64>()
                        .map_err(|_| "packet length missing / not parsable".to_owned())?;
                    let mut packet = String::new();
                    stream
                        // Temporarily clone stream to allow the object to be
                        // moved out of the guard. This is okay as long as we
                        // guarantee that the clone does not outlive the guard.
                        .try_clone()
                        .unwrap()
                        .take(packet_len)
                        .read_to_string(&mut packet)
                        .map_err(|e| e.to_string())?;
                    log::debug!("{}", packet);
                    return serde_json::from_str(&packet)
                        .map(Some)
                        .map_err(|e| e.to_string());
                },
                Ok(1) => buffer.push(buf[0]),
                Ok(_) => unreachable!(),
                Err(e) if e.kind() == ErrorKind::ConnectionReset => return Ok(None), // EOF
                Err(e) => return Err(e.to_string()),
            }
        }
    }
}

/// Wrapper around a client stream that guarantees request/reply invariants.
///
/// Client messages, which are always requests, are dispatched to Actor instances one at a time via
/// [`crate::Actor::handle_message`]. In most cases, each request must be paired with exactly one
/// reply from the same actor the request was sent to, where a reply is a message with no type. (If
/// a message from the server has a type, it’s a notification, not a reply).
///
/// Unless a request is of one of the few types considered "one-way", failing to reply will almost
/// always permanently break that actor, because either the client gets stuck waiting for a reply,
/// or the client receives the reply for a subsequent request as if it was the reply for the current
/// request. If an actor fails to reply to a request, we want the dispatcher
/// ([`crate::ActorRegistry::handle_message`]) to send an error of type `unrecognizedPacketType`,
/// to keep the conversation for that actor in sync.
///
/// Since replies come in all shapes and sizes, we want to allow Actor types to send replies without
/// having to return them to the dispatcher. This wrapper type allows the dispatcher to check if a
/// valid reply was sent, and guarantees that if the actor tries to send a reply, it’s actually a
/// valid reply (see [`Self::is_valid_reply`]).
///
/// It does not currently guarantee anything about messages sent via the [`DevtoolsConnection`]
/// released via [`Self::stream`] or the return value of [`Self::reply`].
pub(crate) struct ClientRequest<'req, 'handled> {
    /// Client stream.
    stream: DevtoolsConnection,
    /// Expected actor name.
    actor_name: &'req str,
    /// Flag allowing ActorRegistry to check for unhandled requests.
    handled: &'handled mut bool,
}

impl ClientRequest<'_, '_> {
    /// Run the given handler, with a new request that wraps the given client stream and expected actor name.
    ///
    /// Returns [`ActorError::UnrecognizedPacketType`] if the actor did not send a reply.
    pub fn handle<'req>(
        stream: DevtoolsConnection,
        actor_name: &'req str,
        handler: impl FnOnce(ClientRequest<'req, '_>) -> Result<(), ActorError>,
    ) -> Result<(), ActorError> {
        let mut sent = false;
        let request = ClientRequest {
            stream,
            actor_name,
            handled: &mut sent,
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
    pub fn reply<T: Serialize>(mut self, reply: &T) -> Result<Self, ActorError> {
        debug_assert!(self.is_valid_reply(reply), "Message is not a valid reply");
        self.stream.write_json_packet(reply)?;
        *self.handled = true;
        Ok(self)
    }

    /// Like `reply`, but for cases where the actor no longer needs the stream.
    pub fn reply_final<T: Serialize>(self, reply: &T) -> Result<(), ActorError> {
        let _stream = self.reply(reply)?;
        Ok(())
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

    /// Manually mark the request as handled, for one-way message types.
    pub fn mark_handled(self) -> Self {
        *self.handled = true;
        self
    }

    /// Get a copy of the client connection.
    pub fn stream(&self) -> DevtoolsConnection {
        self.stream.clone()
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
