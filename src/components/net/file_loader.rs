/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{Metadata, Payload, Done, LoaderTask, start_sending};

use std::io::{ReaderUtil, file_reader};
use std::task;

static READ_SIZE: uint = 1024;

pub fn factory() -> LoaderTask {
    let f: LoaderTask = |url, start_chan| {
        assert!("file" == url.scheme);
        let progress_chan = start_sending(start_chan, Metadata::default(url.clone()));
        do task::spawn {
            match file_reader(&from_str(url.path).unwrap()) {
                Ok(reader) => {
                    while !reader.eof() {
                        let data = reader.read_bytes(READ_SIZE);
                        progress_chan.send(Payload(data));
                    }
                    progress_chan.send(Done(Ok(())));
                }
                Err(*) => {
                    progress_chan.send(Done(Err(())));
                }
            };
        }
    };
    f
}
