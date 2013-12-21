/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io;
use std::io::Writer;
use std::vec::raw::buf_as_slice;
use std::cast::transmute;
use std::mem::size_of;

fn hexdump_slice(buf: &[u8]) {
    let mut stderr = io::stderr();
    stderr.write(bytes!("    "));
    for (i, &v) in buf.iter().enumerate() {
        let output = format!("{:02X} ", v as uint);
        stderr.write(output.as_bytes());
        match i % 16 {
            15 => stderr.write(bytes!("\n    ")),
             7 => stderr.write(bytes!("   ")),
             _ => ()
        }
        stderr.flush();
    }
    stderr.write(bytes!("\n"));
}

pub fn hexdump<T>(obj: &T) {
    unsafe {
        let buf: *u8 = transmute(obj);
        debug!("dumping at {:p}", buf);
        buf_as_slice(buf, size_of::<T>(), hexdump_slice);
    }
}
