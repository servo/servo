/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::comm;
use std::comm::{Chan, Port};
use std::task;

pub fn spawn_listener<A: Send>(f: ~fn(Port<A>)) -> Chan<A> {
    let (setup_po, setup_ch) = comm::stream();
    do task::spawn {
        let (po, ch) = comm::stream();
        setup_ch.send(ch);
        f(po);
    }
    setup_po.recv()
}

pub fn spawn_conversation<A: Send, B: Send>(f: ~fn(Port<A>, Chan<B>)) -> (Port<B>, Chan<A>) {
    let (from_child, to_parent) = comm::stream();
    let to_parent = Cell::new(to_parent);
    let to_child = do spawn_listener |from_parent| {
        f(from_parent, to_parent.take())
    };
    (from_child, to_child)
}
