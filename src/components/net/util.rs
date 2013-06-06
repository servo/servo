/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::comm::{Chan, Port};

pub fn spawn_listener<A: Owned>(f: ~fn(Port<A>)) -> Chan<A> {
    let (setup_port, setup_chan) = comm::stream();
    do task::spawn {
        let (port, chan) = comm::stream();
        setup_chan.send(chan);
        f(port);
    }
    setup_port.recv()
}
