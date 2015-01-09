/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerNavigatorBinding;
use dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::navigatorinfo;
use dom::workerglobalscope::WorkerGlobalScope;
use servo_util::str::DOMString;

#[dom_struct]
pub struct WorkerNavigator {
    reflector_: Reflector,
}

impl WorkerNavigator {
    fn new_inherited() -> WorkerNavigator {
        WorkerNavigator {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: JSRef<WorkerGlobalScope>) -> Temporary<WorkerNavigator> {
        reflect_dom_object(box WorkerNavigator::new_inherited(),
                           GlobalRef::Worker(global),
                           WorkerNavigatorBinding::Wrap)
    }
}

impl<'a> WorkerNavigatorMethods for JSRef<'a, WorkerNavigator> {
    fn Product(self) -> DOMString {
        navigatorinfo::Product()
    }

    fn TaintEnabled(self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    fn AppName(self) -> DOMString {
        navigatorinfo::AppName()
    }

    fn AppCodeName(self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    fn Platform(self) -> DOMString {
        navigatorinfo::Platform()
    }

    fn UserAgent(self) -> DOMString {
        navigatorinfo::UserAgent()
    }
}

