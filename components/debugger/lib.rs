/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate util;
extern crate ws;

use std::sync::mpsc;
use std::sync::mpsc::channel;
use util::thread::spawn_named;
use ws::{Builder, CloseCode, Handler, Handshake, Message};

enum DebuggerMessage {
    ShutdownServer,
}

pub enum DebuggerError {
    SendError
}

type DebuggerResult = Result<(), DebuggerError>;

pub struct DebuggerSender(mpsc::Sender<DebuggerMessage>);

struct DebuggerConnection {
    sender: ws::Sender
}

impl Handler for DebuggerConnection {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        println!("Connection opened.");
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        println!("Connection closed.");
    }

    fn on_message(&mut self, message: Message) -> ws::Result<()> {
        self.sender.send(message)
    }
}

pub fn start_server(port: u16) -> DebuggerSender {
    println!("Starting debugger server.");
    let (sender, receiver) = channel();
    spawn_named("debugger".to_owned(), move || {
        let socket = Builder::new().build(|sender: ws::Sender| {
            DebuggerConnection { sender: sender }
        }).unwrap();
        let sender = socket.broadcaster();
        spawn_named("debugger-websocket".to_owned(), move || {
            socket.listen(("127.0.0.1", port)).unwrap();
        });
        while let Ok(message) = receiver.recv() {
            match message {
                DebuggerMessage::ShutdownServer => {
                    break;
                }
            }
        }
        sender.shutdown().unwrap();
    });
    DebuggerSender(sender)
}

pub fn shutdown_server(sender: &DebuggerSender) -> DebuggerResult {
    println!("Shutting down debugger server.");
    let &DebuggerSender(ref sender) = sender;
    if let Err(_) = sender.send(DebuggerMessage::ShutdownServer) {
        return Err(DebuggerError::SendError)
    }
    Ok(())
}
