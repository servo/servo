/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A task that sniffs data
use std::comm::{channel, Receiver, Sender, Disconnected};
use std::task::TaskBuilder;
use resource_task::{TargetedLoadResponse};

pub type SnifferTask = Sender<TargetedLoadResponse>;

pub fn new_sniffer_task() -> SnifferTask {
    let(sen, rec) = channel();
    let builder = TaskBuilder::new().named("SnifferManager");
    builder.spawn(proc() {
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
            match self.data_receiver.try_recv() {
                Ok(snif_data) => {
                    let result = snif_data.consumer.send_opt(snif_data.load_response);
                    if result.is_err() {
                        break;
                    }
                }
                Err(Disconnected) => break,
                Err(_) => (),
            }
        }
    }
}
