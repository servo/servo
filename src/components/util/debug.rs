/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::io;
use std::vec::raw::buf_as_slice;
use std::cast::transmute;
use std::sys::size_of;

fn hexdump_slice(buf: &[u8]) {
    let stderr = io::stderr();
    stderr.write_str("    ");
    for (i, &v) in buf.iter().enumerate() {
        stderr.write_str(fmt!("%02X ", v as uint));
        match i % 16 {
            15 => stderr.write_str("\n    "),
             7 => stderr.write_str("   "),
             _ => ()
        }
        stderr.flush();
    }
    stderr.write_char('\n');
}

pub fn hexdump<T>(obj: &T) {
    unsafe {
        let buf: *u8 = transmute(obj);
        debug!("dumping at %p", buf);
        buf_as_slice(buf, size_of::<T>(), hexdump_slice);
    }
}
