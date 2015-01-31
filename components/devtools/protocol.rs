/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets]
//! (https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use serialize::{json, Encodable};
use serialize::json::Json;
use std::old_io::{IoError, OtherIoError, EndOfFile, TcpStream, IoResult};
use std::num;

pub trait JsonPacketStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T);
    fn read_json_packet(&mut self) -> IoResult<Json>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<'a, T: Encodable>(&mut self, obj: &T) {
        let s = json::encode(obj).unwrap().replace("__type__", "type");
        println!("<- {}", s);
        self.write_str(s.len().to_string().as_slice()).unwrap();
        self.write_u8(':' as u8).unwrap();
        self.write_str(s.as_slice()).unwrap();
    }

    fn read_json_packet<'a>(&mut self) -> IoResult<Json> {
        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec!();
        loop {
            let colon = ':' as u8;
            match self.read_byte() {
                Ok(c) if c != colon => buffer.push(c as u8),
                Ok(_) => {
                    let packet_len_str = String::from_utf8(buffer).unwrap();
                    let packet_len = num::from_str_radix(packet_len_str.as_slice(), 10).unwrap();
                    let packet_buf = self.read_exact(packet_len).unwrap();
                    let packet = String::from_utf8(packet_buf).unwrap();
                    println!("{}", packet);
                    return Ok(json::from_str(packet.as_slice()).unwrap())
                },
                Err(ref e) if e.kind == EndOfFile =>
                    return Err(IoError { kind: EndOfFile, desc: "EOF", detail: None }),
                _ => return Err(IoError { kind: OtherIoError, desc: "connection error", detail: None })
            }
        }
    }
}
