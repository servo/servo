/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use task_state;
use std::thread;
use std::sync::mpsc::Sender;
use std::thread::Builder;

pub fn spawn_named<F:FnOnce()+Send>(name: String, f: F) {
    let builder = thread::Builder::new().name(name);
    builder.spawn(move || {
        f()
    });
}

/// Arrange to send a particular message to a channel if the task fails.
pub fn spawn_named_with_send_on_failure<F:FnOnce()+Send,T: Send>(name: String,
                                                 state: task_state::TaskState,
                                                 f: F,
                                                 msg: T,
                                                 dest: Sender<T>) {
    let future_handle = thread::Builder::new().name(name.clone()).scoped(move || {
        task_state::initialize(state);
        f()
    });

    let watcher_name = format!("{}Watcher", name);
    Builder::new().name(watcher_name).spawn(move || {
        let future_result = future_handle.join();
        if(future_result.is_err()){
            debug!("{} failed, notifying constellation", name);
            dest.send(msg);
        }
    });
}

#[test]
fn spawn_named_test(){
    spawn_named("Test".to_string(), move || {
        debug!("I can run!");
    });
}
