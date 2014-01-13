/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub fn spawn_listener<A: Send>(f: proc(Port<A>)) -> Chan<A> {
    let (setup_po, setup_ch) = Chan::new();
    spawn(proc() {
        let (po, ch) = Chan::new();
        setup_ch.send(ch);
        f(po);
    });
    setup_po.recv()
}

pub fn spawn_conversation<A: Send, B: Send>(f: proc(Port<A>, Chan<B>)) -> (Port<B>, Chan<A>) {
    let (from_child, to_parent) = Chan::new();
    let to_child = do spawn_listener |from_parent| {
        f(from_parent, to_parent)
    };
    (from_child, to_child)
}
