/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Low-level wire protocol implementation. Currently only supports [JSON packets](https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport#JSON_Packets).

use serialize::{json, Encodable};
use std::io::{IoError, TcpStream};

pub trait JsonPacketSender {
    fn write_json_packet<'a, T: Encodable<json::Encoder<'a>,IoError>>(&mut self, obj: &T);
}

impl JsonPacketSender for TcpStream {
    fn write_json_packet<'a, T: Encodable<json::Encoder<'a>,IoError>>(&mut self, obj: &T) {
        let s = json::encode(obj).replace("__type__", "type");
        println!("<- {:s}", s);
        self.write_str(s.len().to_string().as_slice()).unwrap();
        self.write_u8(':' as u8).unwrap();
        self.write_str(s.as_slice()).unwrap();
    }
}
