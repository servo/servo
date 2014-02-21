/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Metadata, Payload, Done, LoaderTask, start_sending};
use servo_util::io::result;

use std::io;
use std::io::File;
use servo_util::task::spawn_named;

static READ_SIZE: uint = 1024;

fn read_all(reader: &mut io::Stream, progress_chan: &SharedChan<ProgressMsg>)
        -> Result<(), ()> {
    loop {
        match (result(|| {
            let data = reader.read_bytes(READ_SIZE);
            progress_chan.send(Payload(data));
        })) {
            Ok(()) => (),
            Err(e) => match e.kind {
                io::EndOfFile => return Ok(()),
                _ => return Err(()),
            }
        }
    }
}

pub fn factory() -> LoaderTask {
    let f: LoaderTask = proc(url, start_chan) {
        assert!("file" == url.scheme);
        let progress_chan = start_sending(start_chan, Metadata::default(url.clone()));
        spawn_named("file_loader", proc() {
            // ignore_io_error causes us to get None instead of a task failure.
            let _guard = io::ignore_io_error();
            match File::open_mode(&Path::new(url.path), io::Open, io::Read) {
                Some(ref mut reader) => {
                    let res = read_all(reader as &mut io::Stream, &progress_chan);
                    progress_chan.send(Done(res));
                }
                None => {
                    progress_chan.send(Done(Err(())));
                }
            };
        });
    };
    f
}
