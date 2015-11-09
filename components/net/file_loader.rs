/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_task::{ProgressSender, send_error, start_sending_sniffed, start_sending_sniffed_opt};
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
        Ok(0) => Ok(ReadStatus::EOF),
        Ok(n) => {
            buf.truncate(n);
            Ok(ReadStatus::Partial(buf))
        }
        Err(e) => Err(e.description().to_owned()),
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
        let file_path: Result<PathBuf, ()> = url.to_file_path();
        match file_path {
            Ok(file_path) => {
                match File::open(&file_path) {
                    Ok(ref mut reader) => {
                        match read_block(reader) {
                            Ok(ReadStatus::Partial(buf)) => {
                                let metadata = Metadata::default(url);
                                let progress_chan = start_sending_sniffed(senders, metadata,
                                                                          classifier, &buf);
                                progress_chan.send(Payload(buf)).unwrap();
                                let res = read_all(reader, &progress_chan);
                                let _ = progress_chan.send(Done(res));
                            }
                            Ok(ReadStatus::EOF) => {
                                let metadata = Metadata::default(url);
                                if let Ok(chan) = start_sending_sniffed_opt(senders,
                                                                            metadata,
                                                                            classifier,
                                                                            &[]) {
                                    let _ = chan.send(Done(Ok(())));
                                }
                            }
                            Err(e) => {
                                send_error(url, e, senders);
                            }
                        };
                    }
                    Err(e) => {
                        send_error(url, e.description().to_owned(), senders);
                    }
                }
            }
            Err(_) => {
                send_error(url, "Could not parse path".to_owned(), senders);
            }
        }
    });
}
