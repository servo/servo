/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use about_loader;
use mime_classifier::MimeClassifier;
use mime_guess::guess_mime_type;
use msg::constellation_msg::PipelineId;
use net_traits::{LoadConsumer, LoadData, LoadOrigin, Metadata, NetworkError, ReferrerPolicy};
use net_traits::ProgressMsg::{Done, Payload};
use resource_thread::{CancellationListener, ProgressSender};
use resource_thread::{send_error, start_sending_sniffed_opt};
use servo_url::ServoUrl;
use std::borrow::ToOwned;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use util::thread::spawn_named;

static READ_SIZE: usize = 8192;

enum ReadStatus {
    Partial(Vec<u8>),
    EOF,
}

enum LoadResult {
    Cancelled,
    Finished,
}

struct FileLoadOrigin;
impl LoadOrigin for FileLoadOrigin {
    fn referrer_url(&self) -> Option<ServoUrl> {
        None
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        None
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        None
    }
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
    while !cancel_listener.is_cancelled() {
        match try!(read_block(reader)) {
            ReadStatus::Partial(buf) => progress_chan.send(Payload(buf)).unwrap(),
            ReadStatus::EOF => return Ok(LoadResult::Finished),
        }
    }
    let _ = progress_chan.send(Done(Err(NetworkError::LoadCancelled)));
    Ok(LoadResult::Cancelled)
}

fn get_progress_chan(load_data: LoadData, file_path: &Path,
                     senders: LoadConsumer, classifier: Arc<MimeClassifier>, buf: &[u8])
                     -> Result<ProgressSender, ()> {
    let mut metadata = Metadata::default(load_data.url);
    let mime_type = guess_mime_type(file_path);
    metadata.set_content_type(Some(&mime_type));
    return start_sending_sniffed_opt(senders, metadata, classifier, buf, load_data.context);
}

pub fn factory(load_data: LoadData,
               senders: LoadConsumer,
               classifier: Arc<MimeClassifier>,
               cancel_listener: CancellationListener) {
    assert!(load_data.url.scheme() == "file");
    spawn_named("file_loader".to_owned(), move || {
        let file_path = match load_data.url.to_file_path() {
            Ok(file_path) => file_path,
            Err(_) => {
                send_error(load_data.url, NetworkError::Internal("Could not parse path".to_owned()), senders);
                return;
            },
        };
        let mut file = File::open(&file_path);
        let reader = match file {
            Ok(ref mut reader) => reader,
            Err(_) => {
                // this should be one of the three errors listed in
                // http://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.open
                // but, we'll go for a "file not found!"
                let url = ServoUrl::parse("about:not-found").unwrap();
                let load_data_404 = LoadData::new(load_data.context, url, &FileLoadOrigin);
                about_loader::factory(load_data_404, senders, classifier, cancel_listener);
                return;
            }
        };

        if cancel_listener.is_cancelled() {
            if let Ok(progress_chan) = get_progress_chan(load_data, &file_path,
                                                         senders, classifier, &[]) {
                let _ = progress_chan.send(Done(Err(NetworkError::LoadCancelled)));
            }
            return;
        }

        match read_block(reader) {
            Ok(ReadStatus::Partial(buf)) => {
                let progress_chan = get_progress_chan(load_data, &file_path,
                                                      senders, classifier, &buf).ok().unwrap();
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
                if let Ok(chan) = get_progress_chan(load_data, &file_path,
                                                    senders, classifier, &[]) {
                    let _ = chan.send(Done(Ok(())));
                }
            }
            Err(e) => {
                send_error(load_data.url, NetworkError::Internal(e), senders);
            }
        }
    });
}
