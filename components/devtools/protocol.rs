/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets]
//! (https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use rustc_serialize::json::Json;
use rustc_serialize::json::ParserError::{IoError, SyntaxError};
use rustc_serialize::{json, Encodable};
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub trait JsonPacketStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T);
    fn read_json_packet(&mut self) -> Result<Option<Json>, String>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T) {
        let s = json::encode(obj).unwrap().replace("__type__", "type");
        println!("<- {}", s);
        self.write_all(s.len().to_string().as_bytes()).unwrap();
        self.write_all(&[':' as u8]).unwrap();
        self.write_all(s.as_bytes()).unwrap();
    }

    fn read_json_packet<'a>(&mut self) -> Result<Option<Json>, String> {
        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec!();
        loop {
            let mut buf = [0];
            let byte = match self.read(&mut buf) {
                Ok(0) => return Ok(None),  // EOF
                Ok(1) => buf[0],
                Ok(_) => unreachable!(),
                Err(e) => return Err(e.description().to_string()),
            };
            match byte {
                b':' => {
                    let packet_len_str = match String::from_utf8(buffer) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("nonvalid UTF8 in packet length".to_string()),
                    };
                    let packet_len = match u64::from_str_radix(&packet_len_str, 10) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("packet length missing / not parsable".to_string()),
                    };
                    let mut packet = String::new();
                    self.take(packet_len).read_to_string(&mut packet).unwrap();
                    println!("{}", packet);
                    return match Json::from_str(&packet) {
                        Ok(json) => Ok(Some(json)),
                        Err(err) => match err {
                            IoError(ioerr) => return Err(ioerr.description().to_string()),
                            SyntaxError(_, l, c) => return Err(format!("syntax at {}:{}", l, c)),
                        },
                    };
                },
                c => buffer.push(c),
            }
        }
    }
}
