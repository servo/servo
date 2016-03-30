/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets]
//! (https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use serde::Serialize;
use serde_json::{self, Value};
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub trait JsonPacketStream {
    fn write_json_packet<T: Serialize>(&mut self, obj: &T);
    fn read_json_packet(&mut self) -> Result<Option<Value>, String>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<T: Serialize>(&mut self, obj: &T) {
        let s = serde_json::to_string(obj).unwrap();
        println!("<- {}", s);
        write!(self, "{}:{}", s.len(), s).unwrap();
    }

    fn read_json_packet(&mut self) -> Result<Option<Value>, String> {
        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec!();
        loop {
            let mut buf = [0];
            let byte = match self.read(&mut buf) {
                Ok(0) => return Ok(None),  // EOF
                Ok(1) => buf[0],
                Ok(_) => unreachable!(),
                Err(e) => return Err(e.description().to_owned()),
            };
            match byte {
                b':' => {
                    let packet_len_str = match String::from_utf8(buffer) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("nonvalid UTF8 in packet length".to_owned()),
                    };
                    let packet_len = match u64::from_str_radix(&packet_len_str, 10) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("packet length missing / not parsable".to_owned()),
                    };
                    let mut packet = String::new();
                    self.take(packet_len).read_to_string(&mut packet).unwrap();
                    println!("{}", packet);
                    return match serde_json::from_str(&packet) {
                        Ok(json) => Ok(Some(json)),
                        Err(err) => match err {
                            serde_json::Error::Io(ioerr) => {
                                return Err(ioerr.description().to_owned())
                            },
                            serde_json::Error::Syntax(_, l, c) => {
                                return Err(format!("syntax at {}:{}", l, c))
                            },
                            serde_json::Error::FromUtf8(e) => {
                                return Err(e.description().to_owned())
                            },
                        },
                    };
                },
                c => buffer.push(c),
            }
        }
    }
}
