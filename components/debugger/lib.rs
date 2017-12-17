/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate crossbeam_channel;
#[macro_use]
extern crate log;
extern crate ws;

use std::thread;
use ws::{Builder, CloseCode, Handler, Handshake};

enum Message {
    ShutdownServer,
}

pub struct Sender(crossbeam_channel::Sender<Message>);

struct Connection {
    sender: ws::Sender
}

impl Handler for Connection {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        debug!("Connection opened.");
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        debug!("Connection closed.");
    }

    fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
        self.sender.send(message)
    }
}

pub fn start_server(port: u16) -> Sender {
    debug!("Starting server.");
    let (sender, receiver) = crossbeam_channel::unbounded();
    thread::Builder::new().name("debugger".to_owned()).spawn(move || {
        let socket = Builder::new().build(|sender: ws::Sender| {
            Connection { sender: sender }
        }).unwrap();
        let sender = socket.broadcaster();
        thread::Builder::new().name("debugger-websocket".to_owned()).spawn(move || {
            socket.listen(("127.0.0.1", port)).unwrap();
        }).expect("Thread spawning failed");
        while let Some(message) = receiver.recv() {
            match message {
                Message::ShutdownServer => {
                    break;
                }
            }
        }
        sender.shutdown().unwrap();
    }).expect("Thread spawning failed");
    Sender(sender)
}

pub fn shutdown_server(sender: &Sender) {
    debug!("Shutting down server.");
    sender.0.send(Message::ShutdownServer)
}
