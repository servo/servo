/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets]
//! (https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use rustc_serialize::{json, Encodable};
use rustc_serialize::json::Json;
use std::io;
use std::io::{ErrorKind, Read, ReadExt, Write};
use std::net::TcpStream;
use std::num;

pub trait JsonPacketStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T);
    fn read_json_packet(&mut self) -> io::Result<Json>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T) {
        let s = json::encode(obj).unwrap().replace("__type__", "type");
        println!("<- {}", s);
        self.write_all(s.len().to_string().into_bytes().as_slice()).unwrap();
        self.write_all(&[':' as u8]).unwrap();
        self.write_all(s.into_bytes().as_slice()).unwrap();
    }

    fn read_json_packet<'a>(&mut self) -> io::Result<Json> {
        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec!();
        let colon = ':' as u8;
        loop {
            match self.bytes().next() {
                Some(Ok(c)) if c != colon => buffer.push(c as u8),
                Some(Ok(_)) => {
                    let packet_len_str = String::from_utf8(buffer).unwrap();
                    let packet_len = num::from_str_radix(&packet_len_str, 10).unwrap();
                    let mut packet_buf = vec![0; packet_len];
                    self.read(packet_buf.as_mut_slice()).unwrap();
                    let packet = String::from_utf8(packet_buf).unwrap();
                    println!("{}", packet);
                    return Ok(Json::from_str(&packet).unwrap())
                },
                Some(Err(e)) => return Err(io::Error::new(ErrorKind::Other, "connection error", e.detail())),
                None => return Err(io::Error::new(ErrorKind::ConnectionAborted, "EOF", None))
            }
        }
    }
}
