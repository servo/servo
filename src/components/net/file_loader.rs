/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Metadata, Payload, Done, LoaderTask, start_sending};
use servo_util::io::result;

use std::comm::Chan;
use std::rt::io::file;
use std::rt::io::{FileStream, Reader, EndOfFile, Open, Read, ignore_io_error};
use std::task;

static READ_SIZE: uint = 1024;

fn read_all(reader: &mut FileStream, progress_chan: &Chan<ProgressMsg>)
        -> Result<(), ()> {
    loop {
        match (do result {
            let data = reader.read_bytes(READ_SIZE);
            progress_chan.send(Payload(data));
        }) {
            Ok(()) => (),
            Err(e) => match e.kind {
                EndOfFile => return Ok(()),
                _         => return Err(()),
            }
        }
    }
}

pub fn factory() -> LoaderTask {
    let f: LoaderTask = |url, start_chan| {
        assert!("file" == url.scheme);
        let progress_chan = start_sending(start_chan, Metadata::default(url.clone()));
        do task::spawn {
            // ignore_io_error causes us to get None instead of a task failure.
            match ignore_io_error(|| file::open(&url.path.as_slice(), Open, Read)) {
                Some(ref mut reader) => {
                    let res = read_all(reader, &progress_chan);
                    progress_chan.send(Done(res));
                }
                None => {
                    progress_chan.send(Done(Err(())));
                }
            }
        }
    };
    f
}
