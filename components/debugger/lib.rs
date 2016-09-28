/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
extern crate msg;
extern crate util;
extern crate ws;

use msg::constellation_msg::ScriptThreadId;
use std::sync::mpsc;
use std::sync::mpsc::channel;
use util::thread::spawn_named;
use ws::{Builder, CloseCode, Handler, Handshake};

enum Message {
    ShutdownServer,
    ScriptThreadAdded(ScriptThreadId)
}

pub struct Sender(mpsc::Sender<Message>);

struct Connection {
    sender: ws::Sender
}

impl Handler for Connection {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("Connection opened.");
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        println!("Connection closed.");
    }

    fn on_message(&mut self, message: ws::Message) -> ws::Result<()> {
        self.sender.send(message)
    }
}

pub fn start_server(port: u16) -> Sender {
    println!("Starting server.");
    let (sender, receiver) = channel();
    spawn_named("debugger".to_owned(), move || {
        let socket = Builder::new().build(|sender: ws::Sender| {
            Connection { sender: sender }
        }).unwrap();
        let sender = socket.broadcaster();
        spawn_named("debugger-websocket".to_owned(), move || {
            socket.listen(("127.0.0.1", port)).unwrap();
        });
        while let Ok(message) = receiver.recv() {
            match message {
                Message::ShutdownServer => {
                    break;
                }
                Message::ScriptThreadAdded(id) => {

                }
            }
        }
        sender.shutdown().unwrap();
    });
    Sender(sender)
}

pub fn shutdown_server(sender: &Sender) {
    println!("Shutting down server.");
    let &Sender(ref sender) = sender;
    if let Err(_) = sender.send(Message::ShutdownServer) {
        warn!("Failed to shut down server.");
    }
}

pub fn script_thread_added(sender: &Sender, id: ScriptThreadId) {
    println!("Script thread added.");
    let &Sender(ref sender) = sender;
    if let Err(_) = sender.send(Message::ScriptThreadAdded(id)) {
        warn!("");
    }
}
