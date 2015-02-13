/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::old_io as io;
use std::old_io::Writer;
use std::mem;
use std::mem::size_of;
use std::slice;

fn hexdump_slice(buf: &[u8]) {
    let mut stderr = io::stderr();
    stderr.write_all(b"    ").unwrap();
    for (i, &v) in buf.iter().enumerate() {
        let output = format!("{:02X} ", v as uint);
        stderr.write_all(output.as_bytes()).unwrap();
        match i % 16 {
            15 => { stderr.write_all(b"\n    ").unwrap(); },
            7 =>  { stderr.write_all(b"   ").unwrap(); },
             _ => ()
        }
        stderr.flush().unwrap();
    }
    stderr.write_all(b"\n").unwrap();
}

pub fn hexdump<T>(obj: &T) {
    unsafe {
        let buf: *const u8 = mem::transmute(obj);
        debug!("dumping at {:p}", buf);
        let from_buf = slice::from_raw_parts(buf, size_of::<T>());
        hexdump_slice(from_buf);
    }
}
