/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::comm::{Chan, Port};
use std::task;

pub fn spawn_listener<A: Send>(f: proc(Port<A>)) -> SharedChan<A> {
    let (setup_port, setup_chan) = Chan::new();
    do task::spawn {
        let (port, chan) = SharedChan::new();
        setup_chan.send(chan);
        f(port);
    }
    setup_port.recv()
}
