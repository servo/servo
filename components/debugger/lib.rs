/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate util;
extern crate websocket;

use std::sync::mpsc;
use std::sync::mpsc::channel;
use util::thread::spawn_named;
use websocket::{Message, Receiver, Server, WebSocketStream};
use websocket::message::Type;
use websocket::server::Connection;

enum DebuggerMessage {
    ShutdownServer,
    ConnectionAccepted(Connection<WebSocketStream, WebSocketStream>)
}

pub enum DebuggerError {
    SendError
}

pub struct DebuggerMessageSender(mpsc::Sender<DebuggerMessage>);

pub fn start_server(port: u16) -> DebuggerMessageSender {
    println!("Starting debugger server.");
    let (sender, receiver) = channel();
    {
        let sender = sender.clone();
        spawn_named("debugger".to_owned(), move || {
            run_server(port, sender, receiver)
        });
    }
    DebuggerMessageSender(sender)
}

pub fn shutdown_server(sender: &DebuggerMessageSender) -> Result<(), DebuggerError> {
    println!("Shutting down debugger server.");
    let &DebuggerMessageSender(ref sender) = sender;
    if let Err(_) = sender.send(DebuggerMessage::ShutdownServer) {
        return Err(DebuggerError::SendError)
    }
    Ok(())
}

fn run_server(port: u16, sender: mpsc::Sender<DebuggerMessage>, receiver: mpsc::Receiver<DebuggerMessage>) {
    let server = Server::bind(("127.0.0.1", port)).unwrap();
    spawn_named("debugger-connection-acceptor".to_owned(), move || {
        for connection in server {
            sender.send(DebuggerMessage::ConnectionAccepted(connection.unwrap())).unwrap();
        }
    });
    while let Ok(message) = receiver.recv() {
        match message {
            DebuggerMessage::ShutdownServer => {
                break;
            }
            DebuggerMessage::ConnectionAccepted(connection) => {
                spawn_named("debugger-connection-handler".to_owned(), move || {
                    handle_connection(connection);
                });
            }
        }
    }
}

fn handle_connection(connection: Connection<WebSocketStream, WebSocketStream>) {
    let request = connection.read_request().unwrap();
    let response = request.accept();
    let client = response.send().unwrap();
    let (mut sender, mut receiver) = client.split();
    for message in receiver.incoming_messages() {
        let message: Message = message.unwrap();
        match message.opcode {
            Type::Close => {
                let message = Message::close();
                websocket::Sender::send_message(&mut sender, &message).unwrap();
                break;
            }
            Type::Ping => {
                let message = Message::pong(message.payload);
                websocket::Sender::send_message(&mut sender, &message).unwrap();
            }
            Type::Text => {
                websocket::Sender::send_message(&mut sender, &message).unwrap();
            }
            _ => {
                panic!("Unexpected message type.");
            }
        }
    }
}
