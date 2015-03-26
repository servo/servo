/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Metadata, LoadData, start_sending, TargetedLoadResponse, ResponseSenders};
use resource_task::ProgressMsg::{Payload, Done};

use std::borrow::ToOwned;
use std::io;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use util::task::spawn_named;

static READ_SIZE: uint = 8192;

fn read_all(reader: &mut io::Read, progress_chan: &Sender<ProgressMsg>)
        -> Result<(), String> {
    loop {
        let mut buf = vec![0; READ_SIZE];
        match reader.read(buf.as_mut_slice()) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                buf.truncate(n);
                progress_chan.send(Payload(buf)).unwrap();
            },
            Err(e) => return Err(e.description().to_string()),
        }
    }
}

pub fn factory(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    let url = load_data.url;
    assert!(&*url.scheme == "file");
    let senders = ResponseSenders {
        immediate_consumer: start_chan,
        eventual_consumer: load_data.consumer,
    };
    let progress_chan = start_sending(senders, Metadata::default(url.clone()));
    spawn_named("file_loader".to_owned(), move || {
        let file_path: Result<PathBuf, ()> = url.to_file_path();
        match file_path {
            Ok(file_path) => {
                match File::open(&file_path) {
                    Ok(ref mut reader) => {
                        let res = read_all(reader, &progress_chan);
                        progress_chan.send(Done(res)).unwrap();
                    }
                    Err(e) => {
                        progress_chan.send(Done(Err(e.description().to_string()))).unwrap();
                    }
                }
            }
            Err(_) => {
                progress_chan.send(Done(Err(url.to_string()))).unwrap();
            }
        }
    });
}
