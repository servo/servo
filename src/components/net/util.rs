/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/*use std::comm::{Chan, Port};
use servo_util::task::spawn_named;*/


    // FIXME: code cloned from spawn_listener due to:
    //  error: internal compiler error: cannot relate bound region: ReLateBound(6270, BrNamed(syntax::ast::DefId{krate: 0u32, node: 6294u32}, a)) <= ReInfer(1)
    //This message reflects a bug in the Rust compiler. 

/*
pub fn spawn_listener<'a, A: Send, S: IntoMaybeOwned<'a>>(name: S, f: proc(Port<A>)) -> Chan<A> {
    let (setup_port, setup_chan) = Chan::new();
    spawn_named(name, proc() {
        let (port, chan) = Chan::new();
        setup_chan.send(chan);
        f(port);
    });
    setup_port.recv()
}
*/