/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerNavigatorBinding;
use dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use dom::bindings::global::Worker;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::workerglobalscope::WorkerGlobalScope;
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct WorkerNavigator {
    reflector_: Reflector,
}

impl WorkerNavigator {
    pub fn new_inherited() -> WorkerNavigator {
        WorkerNavigator {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: JSRef<WorkerGlobalScope>) -> Temporary<WorkerNavigator> {
        reflect_dom_object(box WorkerNavigator::new_inherited(),
                           &Worker(global),
                           WorkerNavigatorBinding::Wrap)
    }
}

impl<'a> WorkerNavigatorMethods for JSRef<'a, WorkerNavigator> {
    fn Product(&self) -> DOMString {
        "Gecko".to_string()
    }

    fn TaintEnabled(&self) -> bool {
        false
    }

    fn AppName(&self) -> DOMString {
        "Netscape".to_string() // Like Gecko/Webkit
    }

    fn AppCodeName(&self) -> DOMString {
        "Mozilla".to_string()
    }

    fn Platform(&self) -> DOMString {
        "".to_string()
    }
}

impl Reflectable for WorkerNavigator {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
