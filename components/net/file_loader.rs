/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Payload, Done};
use net_traits::{LoadData, Metadata, LoadConsumer};
use resource_task::{start_sending, start_sending_sniffed, ProgressSender};

use std::borrow::ToOwned;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use util::task::spawn_named;

static READ_SIZE: usize = 8192;

enum ReadStatus {
    Partial(Vec<u8>),
    EOF,
}

fn read_block(reader: &mut File) -> Result<ReadStatus, String> {
    let mut buf = vec![0; READ_SIZE];
    match reader.read(&mut buf) {
        Ok(0) => return Ok(ReadStatus::EOF),
        Ok(n) => {
            buf.truncate(n);
            Ok(ReadStatus::Partial(buf))
        }
        Err(e) => Err(e.description().to_string()),
    }
}

fn read_all(reader: &mut File, progress_chan: &ProgressSender)
            -> Result<(), String> {
    loop {
        match try!(read_block(reader)) {
            ReadStatus::Partial(buf) => progress_chan.send(Payload(buf)).unwrap(),
            ReadStatus::EOF => return Ok(()),
        }
    }
}

pub fn factory(load_data: LoadData, senders: LoadConsumer, classifier: Arc<MIMEClassifier>) {
    let url = load_data.url;
    assert!(&*url.scheme == "file");
    spawn_named("file_loader".to_owned(), move || {
        let metadata = Metadata::default(url.clone());
        let file_path: Result<PathBuf, ()> = url.to_file_path();
        match file_path {
            Ok(file_path) => {
                match File::open(&file_path) {
                    Ok(ref mut reader) => {
                        let res = read_block(reader);
                        let (res, progress_chan) = match res {
                            Ok(ReadStatus::Partial(buf)) => {
                                let progress_chan = start_sending_sniffed(senders, metadata,
                                                                          classifier, &buf);
                                progress_chan.send(Payload(buf)).unwrap();
                                (read_all(reader, &progress_chan), progress_chan)
                            }
                            Ok(ReadStatus::EOF) | Err(_) =>
                                (res.map(|_| ()), start_sending(senders, metadata)),
                        };
                        progress_chan.send(Done(res)).unwrap();
                    }
                    Err(e) => {
                        let progress_chan = start_sending(senders, metadata);
                        progress_chan.send(Done(Err(e.description().to_string()))).unwrap();
                    }
                }
            }
            Err(_) => {
                let progress_chan = start_sending(senders, metadata);
                progress_chan.send(Done(Err(url.to_string()))).unwrap();
            }
        }
    });
}
