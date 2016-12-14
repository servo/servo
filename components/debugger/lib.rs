/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use]
extern crate log;
#[cfg(not(target_os = "android"))]
extern crate ws;

use std::sync::mpsc;
use std::sync::mpsc::channel;
use std::thread;
#[cfg(not(target_os = "android"))]
use ws::{Builder, CloseCode, Handler, Handshake};

enum Message {
    ShutdownServer,
}

pub struct Sender(mpsc::Sender<Message>);

#[cfg(not(target_os = "android"))]
struct Connection {
    sender: ws::Sender
}

#[cfg(not(target_os = "android"))]
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

#[cfg(not(target_os = "android"))]
pub fn start_server(port: u16) -> Sender {
    debug!("Starting server.");
    let (sender, receiver) = channel();
    thread::Builder::new().name("debugger".to_owned()).spawn(move || {
        let socket = Builder::new().build(|sender: ws::Sender| {
            Connection { sender: sender }
        }).unwrap();
        let sender = socket.broadcaster();
        thread::Builder::new().name("debugger-websocket".to_owned()).spawn(move || {
            socket.listen(("127.0.0.1", port)).unwrap();
        }).expect("Thread spawning failed");
        while let Ok(message) = receiver.recv() {
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

#[cfg(target_os = "android")]
pub fn start_server(_: u16) -> Sender {
    panic!("Debugger is not supported on Android");
}

#[cfg(not(target_os = "android"))]
pub fn shutdown_server(sender: &Sender) {
    debug!("Shutting down server.");
    let &Sender(ref sender) = sender;
    if let Err(_) = sender.send(Message::ShutdownServer) {
        warn!("Failed to shut down server.");
    }
}

#[cfg(target_os = "android")]
pub fn shutdown_server(_: &Sender) {
    panic!("Debugger is not supported on Android");
}
