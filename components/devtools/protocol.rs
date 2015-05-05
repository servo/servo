/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets]
//! (https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use rustc_serialize::{json, Encodable};
use rustc_serialize::json::Json;
use std::io::{self, Read, Write};
use std::net::TcpStream;

pub trait JsonPacketStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T);
    fn read_json_packet(&mut self) -> io::Result<Option<Json>>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T) {
        let s = json::encode(obj).unwrap().replace("__type__", "type");
        println!("<- {}", s);
        self.write_all(s.len().to_string().as_bytes()).unwrap();
        self.write_all(&[':' as u8]).unwrap();
        self.write_all(s.as_bytes()).unwrap();
    }

    fn read_json_packet<'a>(&mut self) -> io::Result<Option<Json>> {
        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec!();
        loop {
            let mut buf = [0];
            let byte = match try!(self.read(&mut buf)) {
                0 => return Ok(None),  // EOF
                1 => buf[0],
                _ => unreachable!(),
            };
            match byte {
                b':' => {
                    let packet_len_str = String::from_utf8(buffer).unwrap();
                    let packet_len = u64::from_str_radix(&packet_len_str, 10).unwrap();
                    let mut packet = String::new();
                    self.take(packet_len).read_to_string(&mut packet).unwrap();
                    println!("{}", packet);
                    return Ok(Some(Json::from_str(&packet).unwrap()))
                },
                c => buffer.push(c),
            }
        }
    }
}
