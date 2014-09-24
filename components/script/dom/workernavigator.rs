/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerNavigatorBinding;
use dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use dom::bindings::global::Worker;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::navigatorinfo::NavigatorInfo;
use dom::workerglobalscope::WorkerGlobalScope;
use servo_util::str::DOMString;

#[jstraceable]
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
    fn Product(self) -> DOMString {
        NavigatorInfo::Product()
    }

    fn TaintEnabled(self) -> bool {
        NavigatorInfo::TaintEnabled()
    }

    fn AppName(self) -> DOMString {
        NavigatorInfo::AppName()
    }

    fn AppCodeName(self) -> DOMString {
        NavigatorInfo::AppCodeName()
    }

    fn Platform(self) -> DOMString {
        NavigatorInfo::Platform()
    }
}

impl Reflectable for WorkerNavigator {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
