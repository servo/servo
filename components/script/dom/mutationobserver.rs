/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::ptr::null;
use dom;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationCallback;
use dom::bindings::codegen::Bindings::MutationObserverBinding::MutationObserverInit;
use dom::bindings::error::Fallible;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::trace::JSTraceable;
use dom::mutationrecord::MutationRecord;
use dom::node::Node;
use dom::window::Window;
use dom_struct::dom_struct;
use script_thread::ScriptThread;
use std::ops::Deref;
use std::rc::Rc;

#[dom_struct]
pub struct MutationObserver {
    reflector_: Reflector,
    callback: MutationCallback,
}

impl MutationObserver {
    fn new(global: &Window, callback: Rc<MutationCallback>) -> MutationObserver {
        MutationObserver {
            reflector_: Reflector::new(),
            callback: callback,
        }
    }

    pub fn Constructor(global: &Window, callback: Rc<MutationCallback>) -> Fallible<Root<MutationObserver>> {
        let observer = MutationObserver::new(global, callback);
        ScriptThread::add_mutation_observer(Root::from_ref(&*observer))
    }

}
