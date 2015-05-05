/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// A quick hack to work around the removal of [`std::old_io::timer::Timer`](
/// http://doc.rust-lang.org/1.0.0-beta/std/old_io/timer/struct.Timer.html )

use std::sync::mpsc::{channel, Receiver};
use std::thread::{spawn, sleep_ms};

pub fn oneshot(duration_ms: u32) -> Receiver<()> {
    let (tx, rx) = channel();
    spawn(move || {
        sleep_ms(duration_ms);
        let _ = tx.send(());
    });
    rx
}

pub fn periodic(duration_ms: u32) -> Receiver<()> {
    let (tx, rx) = channel();
    spawn(move || {
        loop {
            sleep_ms(duration_ms);
            if tx.send(()).is_err() {
                break
            }
        }
    });
    rx
}
