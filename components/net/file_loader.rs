/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use resource_task::{ProgressMsg, Metadata, LoadData, start_sending, TargetedLoadResponse, ResponseSenders};
use resource_task::ProgressMsg::{Payload, Done};

use std::borrow::ToOwned;
use std::old_io as io;
use std::old_io::File;
use std::sync::mpsc::Sender;
use util::task::spawn_named;

static READ_SIZE: uint = 8192;

fn read_all(reader: &mut io::Stream, progress_chan: &Sender<ProgressMsg>)
        -> Result<(), String> {
    loop {
        let mut buf = vec!();
        match reader.push_at_least(READ_SIZE, READ_SIZE, &mut buf) {
            Ok(_) => progress_chan.send(Payload(buf)).unwrap(),
            Err(e) => match e.kind {
                io::EndOfFile => {
                    if buf.len() > 0 {
                        progress_chan.send(Payload(buf)).unwrap();
                    }
                    return Ok(());
                }
                _ => return Err(e.desc.to_string()),
            }
        }
    }
}

pub fn factory(load_data: LoadData, start_chan: Sender<TargetedLoadResponse>) {
    let url = load_data.url;
    assert!("file" == url.scheme.as_slice());
    let senders = ResponseSenders {
        immediate_consumer: start_chan,
        eventual_consumer: load_data.consumer,
    };
    let progress_chan = start_sending(senders, Metadata::default(url.clone()));
    spawn_named("file_loader".to_owned(), move || {
        let file_path: Result<Path, ()> = url.to_file_path();
        match file_path {
            Ok(file_path) => {
                match File::open_mode(&Path::new(file_path), io::Open, io::Read) {
                    Ok(ref mut reader) => {
                        let res = read_all(reader as &mut io::Stream, &progress_chan);
                        progress_chan.send(Done(res)).unwrap();
                    }
                    Err(e) => {
                        progress_chan.send(Done(Err(e.desc.to_string()))).unwrap();
                    }
                }
            }
            Err(_) => {
                progress_chan.send(Done(Err(url.to_string()))).unwrap();
            }
        }
    });
}
