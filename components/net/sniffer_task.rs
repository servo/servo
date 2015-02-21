/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::Builder;
use mime_classifier::MIMEClassifier;
use resource_task::{LoadResponse, LoadResponse, ProgressMsg};

pub type SnifferTask = Sender<TargetedLoadResponse>;

pub fn new_sniffer_task() -> SnifferTask {
    let(sen, rec) = channel();
    let builder = Builder::new().name("SnifferManager".to_string());
    builder.spawn(move || {
        SnifferManager::new(rec).start();
    }).unwrap();
    sen
}

struct SnifferManager {
    data_receiver: Receiver<TargetedLoadResponse>,
    mime_classifier: MIMEClassifier
}

impl SnifferManager {
    fn new(data_receiver: Receiver <TargetedLoadResponse>) -> SnifferManager {
        SnifferManager {
            data_receiver: data_receiver,
	    mime_classifier: MIMEClassifier::new()
        }
    }
}

impl SnifferManager {
    fn start(self) {

        for mut snif_data in self.data_receiver.iter() {
            // Read all the data
            let mut resource_data = vec!();
            loop {
                match snif_data.load_response.progress_port.recv().unwrap() {
                    ProgressMsg::Payload(data) => {
                        resource_data.push_all(data.as_slice());
                    }
                    ProgressMsg::Done(res) => {
                        let (new_progress_chan, new_progress_port) = channel();

                        // TODO: should be calculated in the resource loader, from pull requeset #4094
                        let nosniff = false;
                        let check_for_apache_bug = false;

                        // We have all the data, go ahead and sniff it and replace the Content-Type
                        if res.is_ok() {
                            snif_data.load_response.metadata.content_type = self.mime_classifier.classify(
                                nosniff,check_for_apache_bug,&snif_data.load_response.metadata.content_type,
                                &resource_data
                            );
                        }
                        let load_response = LoadResponse {
                            progress_port: new_progress_port,
                            metadata: snif_data.load_response.metadata,
                        };

                        if snif_data.consumer.send(load_response).is_err() {
                            break;
                        }
                        if resource_data.len() > 0 {
                            new_progress_chan.send(ProgressMsg::Payload(resource_data)).unwrap();
                        }
                        new_progress_chan.send(ProgressMsg::Done(res)).unwrap();
                        return;
                    }
                }
            }
        } // end for
    }
}

#[cfg(test)]
pub fn new_mock_sniffer_task() -> SnifferTask {
    let(sen, rec) = channel();
    let builder = TaskBuilder::new().named("SnifferManager");
    builder.spawn(move || {
        MockSnifferManager::new(rec).start();
    });
    sen
}

#[cfg(test)]
struct MockSnifferManager {
    data_receiver: Receiver<TargetedLoadResponse>,
}

#[cfg(test)]
impl MockSnifferManager {
    fn new(data_receiver: Receiver <TargetedLoadResponse>) -> MockSnifferManager {
        MockSnifferManager {
            data_receiver: data_receiver,
        }
    }
}

#[cfg(test)]
impl MockSnifferManager {
    fn start(self) {
        loop {
            match self.data_receiver.recv() {
                Ok(snif_data) => {
                    let _ = snif_data.consumer.send(snif_data.load_response);
                }
                Err(_) => break,
            }
        }
    }
}
