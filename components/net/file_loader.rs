/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use net_traits::{LoadData, Metadata};
use mime_classifier::MIMEClassifier;
use resource_task::ResourceConsumer;

use std::borrow::ToOwned;
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
    match reader.read(buf.as_mut_slice()) {
        Ok(0) => return Ok(ReadStatus::EOF),
        Ok(n) => {
            buf.truncate(n);
            Ok(ReadStatus::Partial(buf))
        }
        Err(e) => Err(e.description().to_string()),
    }
}

fn read_all(reader: &mut File, resource_consumer: &mut ResourceConsumer)
            -> Result<(), String> {
    loop {
        match try!(read_block(reader)) {
            ReadStatus::Partial(buf) => resource_consumer.send(buf),
            ReadStatus::EOF => return Ok(()),
        }
    }
}

pub fn factory(mut resource_consumer: ResourceConsumer, load_data: LoadData, classifier: Arc<MIMEClassifier>) {
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
                        let res = match res {
                            Ok(ReadStatus::Partial(buf)) => {
                                resource_consumer.start_sniffed(metadata,
                                                            classifier, &buf);
                                resource_consumer.send(buf);
                                read_all(reader, &mut resource_consumer)
                            }
                            Ok(ReadStatus::EOF) | Err(_) => {
                                resource_consumer.start(metadata);
                                res.map(|_| ())
                            }
                        };
                        resource_consumer.done(res);
                    }
                    Err(e) => {
                        resource_consumer.start(metadata);
                        resource_consumer.error(e.description().to_string());
                    }
                }
            }
            Err(_) => {
                resource_consumer.start(metadata);
                resource_consumer.error(url.to_string());
            }
        }
    });
}
