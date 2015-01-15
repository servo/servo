/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::Builder;
use resource_task::{TargetedLoadResponse};

pub type SnifferTask = Sender<TargetedLoadResponse>;

pub fn new_sniffer_task() -> SnifferTask {
    let(sen, rec) = channel();
    let builder = Builder::new().name("SnifferManager".to_string());
    builder.spawn(move || {
        SnifferManager::new(rec).start();
    });
    sen
}

struct SnifferManager {
    data_receiver: Receiver<TargetedLoadResponse>,
}

impl SnifferManager {
    fn new(data_receiver: Receiver <TargetedLoadResponse>) -> SnifferManager {
        SnifferManager {
            data_receiver: data_receiver,
        }
    }
}

impl SnifferManager {
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
