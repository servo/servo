/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mime_classifier::MIMEClassifier;
use mime_guess::guess_mime_type;
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_task::{CancellationListener, ProgressSender};
use resource_task::{send_error, start_sending_sniffed, start_sending_sniffed_opt};
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

enum LoadResult {
    Cancelled,
    Finished,
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

fn read_all(reader: &mut File, progress_chan: &ProgressSender, cancel_listener: &CancellationListener)
            -> Result<LoadResult, String> {
    loop {
        if cancel_listener.is_cancelled() {
            return Ok(LoadResult::Cancelled);
        }

        match try!(read_block(reader)) {
            ReadStatus::Partial(buf) => progress_chan.send(Payload(buf)).unwrap(),
            ReadStatus::EOF => return Ok(LoadResult::Finished),
        }
    }
}

pub fn factory(load_data: LoadData,
               senders: LoadConsumer,
               classifier: Arc<MIMEClassifier>,
               cancel_listener: CancellationListener) {
    let url = load_data.url;
    assert!(&*url.scheme == "file");
    spawn_named("file_loader".to_owned(), move || {
        let file_path: Result<PathBuf, ()> = url.to_file_path();
        match file_path {
            Ok(file_path) => {
                match File::open(&file_path) {
                    Ok(ref mut reader) => {
                        if cancel_listener.is_cancelled() {
                            return;
                        }
                        match read_block(reader) {
                            Ok(ReadStatus::Partial(buf)) => {
                                let mut metadata = Metadata::default(url);
                                let mime_type = guess_mime_type(file_path.as_path());
                                metadata.set_content_type(Some(&mime_type));
                                let progress_chan = start_sending_sniffed(senders, metadata,
                                                                          classifier, &buf);
                                progress_chan.send(Payload(buf)).unwrap();
                                let read_result = read_all(reader, &progress_chan, &cancel_listener);
                                if let Ok(load_result) = read_result {
                                    match load_result {
                                        LoadResult::Cancelled => return,
                                        LoadResult::Finished => progress_chan.send(Done(Ok(()))).unwrap(),
                                    }
                                }
                            }
                            Ok(ReadStatus::EOF) => {
                                let mut metadata = Metadata::default(url);
                                let mime_type = guess_mime_type(file_path.as_path());
                                metadata.set_content_type(Some(&mime_type));
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
