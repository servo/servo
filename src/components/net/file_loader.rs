/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Metadata, Payload, Done, LoaderTask, start_sending};

use std::io;
use std::io::File;
use servo_util::task::spawn_named;

//FIXME: https://github.com/mozilla/rust/issues/12892
static READ_SIZE: uint = 1;

fn read_all(reader: &mut io::Stream, progress_chan: &Sender<ProgressMsg>)
        -> Result<(), String> {
    loop {
        let mut buf = vec!();
        match reader.push_at_least(READ_SIZE, READ_SIZE, &mut buf) {
            Ok(_) => progress_chan.send(Payload(buf)),
            Err(e) => match e.kind {
                io::EndOfFile => return Ok(()),
                _ => return Err(e.desc.to_string()),
            }
        }
    }
}

pub fn factory() -> LoaderTask {
    let f: LoaderTask = proc(load_data, start_chan) {
        let url = load_data.url;
        assert!("file" == url.scheme.as_slice());
        let progress_chan = start_sending(start_chan, Metadata::default(url.clone()));
        spawn_named("file_loader", proc() {
            match File::open_mode(&Path::new(url.serialize_path().unwrap()), io::Open, io::Read) {
                Ok(ref mut reader) => {
                    let res = read_all(reader as &mut io::Stream, &progress_chan);
                    progress_chan.send(Done(res));
                }
                Err(e) => {
                    progress_chan.send(Done(Err(e.desc.to_string())));
                }
            };
        });
    };
    f
}
