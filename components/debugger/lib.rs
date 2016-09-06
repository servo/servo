/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate util;
extern crate websocket;

use util::thread::spawn_named;
use websocket::{Message, Receiver, Sender, Server};
use websocket::message::Type;

pub fn start_server(port: u16) {
    println!("Starting debugger server.");
    spawn_named("debugger-server".to_owned(), move || {
        run_server(port)
    });
}

fn run_server(port: u16) {
    let server = Server::bind(("127.0.0.1", port)).unwrap();
    for connection in server {
        spawn_named("debugger-connection".to_owned(), move || {
            let connection = connection.unwrap();
            let request = connection.read_request().unwrap();
            let response = request.accept();
            let client = response.send().unwrap();
            let (mut sender, mut receiver) = client.split();
            for message in receiver.incoming_messages() {
                let message: Message = message.unwrap();
                match message.opcode {
                    Type::Close => {
                        let message = Message::close();
                        sender.send_message(&message).unwrap();
                        break;
                    }
                    Type::Ping => {
                        let message = Message::pong(message.payload);
                        sender.send_message(&message).unwrap();
                    }
                    Type::Text => {
                        sender.send_message(&message).unwrap();
                    }
                    _ => {
                        panic!("Unexpected message type.");
                    }
                }
            }
        });
    }
}
