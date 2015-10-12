/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mime_classifier::MIMEClassifier;
use net_traits::ProgressMsg::{Done, Payload};
use net_traits::{LoadConsumer, LoadData, Metadata};
use resource_task::{send_error, start_sending};
use std::borrow::ToOwned;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use url::Url;
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

pub fn factory(load_data: LoadData, senders: LoadConsumer, classifier: Arc<MIMEClassifier>) {
    let url = load_data.url;
    assert!(&*url.scheme == "file");
    spawn_named("file_loader".to_owned(), move || load(url, senders, classifier));
}

fn load(url: Url, senders: LoadConsumer, classifier: Arc<MIMEClassifier>) {
    // Open the file.
    let reader = url.to_file_path()
        .map_err(|_| "Cannot parse URL".to_owned())
        .and_then(|path| File::open(&path).map_err(|e| e.to_string()));
    let mut reader = match reader {
        Ok(reader) => reader,
        Err(e) => {
            send_error(url, e, senders);
            return;
        }
    };

    // Start reading the file.
    match read_block(&mut reader) {
        Ok(ReadStatus::Partial(buf)) => {
            // We read the first chunk of the file; start sending it across the
            // channel.
            // FIXME: Unlikely to happen in practice, but theoretically
            // `buf` might not contain enough bytes for sniffing to be
            // deterministic.
            let metadata = Metadata::default(url);
            if let Ok(progress_chan) = start_sending(senders, metadata, classifier, &buf) {
                if progress_chan.send(Payload(buf)).is_err() {
                    return
                }
                loop {
                    match read_block(&mut reader) {
                        Ok(ReadStatus::Partial(buf)) => {
                            if progress_chan.send(Payload(buf)).is_err() {
                                break;
                            }
                        }
                        Ok(ReadStatus::EOF) => {
                            let _ = progress_chan.send(Done(Ok(())));
                            break;
                        }
                        Err(e) => {
                            let _ = progress_chan.send(Done(Err(e)));
                            break;
                        }
                    }
                }
            }
        }
        Ok(ReadStatus::EOF) => {
            // This is sort of a weird special-case: we successfully read the
            // file, but it was empty.
            let metadata = Metadata::default(url);
            if let Ok(progress_chan) = start_sending(senders, metadata, classifier, &[]) {
                let _ = progress_chan.send(Done(Ok(())));
            }
        }
        Err(e) => send_error(url, e, senders),
    };
}
