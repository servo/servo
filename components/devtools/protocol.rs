/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Low-level wire protocol implementation. Currently only supports
//! [JSON packets](https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#json-packets).

use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

use log::debug;
use serde::Serialize;
use serde_json::{self, Value};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorDescription {
    pub category: &'static str,
    pub type_name: &'static str,
    pub methods: Vec<Method>,
}

#[derive(Serialize)]
pub struct Method {
    pub name: &'static str,
    pub request: Value,
    pub response: Value,
}

pub trait JsonPacketStream {
    fn write_json_packet<T: Serialize>(&mut self, obj: &T) -> Result<(), Box<dyn Error>>;
    fn write_merged_json_packet<T: Serialize, U: Serialize>(
        &mut self,
        base: &T,
        extra: &U,
    ) -> Result<(), Box<dyn Error>>;
    fn read_json_packet(&mut self) -> Result<Option<Value>, String>;
}

impl JsonPacketStream for TcpStream {
    fn write_json_packet<T: Serialize>(&mut self, obj: &T) -> Result<(), Box<dyn Error>> {
        let s = serde_json::to_string(obj)?;
        debug!("<- {}", s);
        write!(self, "{}:{}", s.len(), s)?;
        Ok(())
    }

    fn write_merged_json_packet<T: Serialize, U: Serialize>(
        &mut self,
        base: &T,
        extra: &U,
    ) -> Result<(), Box<dyn Error>> {
        let mut obj = serde_json::to_value(base)?;
        let obj = obj.as_object_mut().unwrap();
        let extra = serde_json::to_value(extra)?;
        let extra = extra.as_object().unwrap();

        for (key, value) in extra {
            obj.insert(key.to_owned(), value.to_owned());
        }

        self.write_json_packet(obj)
    }

    fn read_json_packet(&mut self) -> Result<Option<Value>, String> {
        // https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#stream-transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        let mut buffer = vec![];
        loop {
            let mut buf = [0];
            let byte = match self.read(&mut buf) {
                Ok(0) => return Ok(None), // EOF
                Ok(1) => buf[0],
                Ok(_) => unreachable!(),
                Err(e) => return Err(e.to_string()),
            };
            match byte {
                b':' => {
                    let packet_len_str = match String::from_utf8(buffer) {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("nonvalid UTF8 in packet length".to_owned()),
                    };
                    let packet_len = match packet_len_str.parse::<u64>() {
                        Ok(packet_len) => packet_len,
                        Err(_) => return Err("packet length missing / not parsable".to_owned()),
                    };
                    let mut packet = String::new();
                    self.take(packet_len)
                        .read_to_string(&mut packet)
                        .map_err(|e| e.to_string())?;
                    debug!("{}", packet);
                    return match serde_json::from_str(&packet) {
                        Ok(json) => Ok(Some(json)),
                        Err(err) => Err(err.to_string()),
                    };
                },
                c => buffer.push(c),
            }
        }
    }
}
