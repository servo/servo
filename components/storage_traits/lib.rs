/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(custom_derive)]
#![feature(plugin)]
#![plugin(heapsize_plugin, serde_macros)]

#[macro_use]
extern crate heapsize;
extern crate ipc_channel;
extern crate rustc_serialize;
#[macro_use]
extern crate serde;
extern crate url;

pub mod storage_thread;

use rustc_serialize::Encodable;
use rustc_serialize::json;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub fn write_json_to_file<T: Encodable>(data: &T, profile_dir: &str, filename: &str) {
    let json_encoded: String;
    match json::encode(&data) {
        Ok(d) => json_encoded = d,
        Err(_) => return,
    }
    let path = Path::new(profile_dir).join(filename);
    let display = path.display();

    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           Error::description(&why)),
        Ok(file) => file,
    };

    match file.write_all(json_encoded.as_bytes()) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               Error::description(&why))
        },
        Ok(_) => println!("successfully wrote to {}", display),
    }
}
