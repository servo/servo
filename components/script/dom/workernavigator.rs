/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerNavigatorBinding;
use dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::navigatorinfo;
use dom::workerglobalscope::WorkerGlobalScope;
use util::str::DOMString;

// https://html.spec.whatwg.org/multipage/#workernavigator
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct WorkerNavigator {
    reflector_: Reflector,
}

impl WorkerNavigator {
    fn new_inherited() -> WorkerNavigator {
        WorkerNavigator {
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &WorkerGlobalScope) -> Root<WorkerNavigator> {
        reflect_dom_object(box WorkerNavigator::new_inherited(),
                           GlobalRef::Worker(global),
                           WorkerNavigatorBinding::Wrap)
    }
}

impl<'a> WorkerNavigatorMethods for &'a WorkerNavigator {
    // https://html.spec.whatwg.org/multipage/#dom-navigator-product
    fn Product(self) -> DOMString {
        navigatorinfo::Product()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-taintenabled
    fn TaintEnabled(self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appname
    fn AppName(self) -> DOMString {
        navigatorinfo::AppName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appcodename
    fn AppCodeName(self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-platform
    fn Platform(self) -> DOMString {
        navigatorinfo::Platform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-useragent
    fn UserAgent(self) -> DOMString {
        navigatorinfo::UserAgent()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appversion
    fn AppVersion(self) -> DOMString {
        navigatorinfo::AppVersion()
    }
}

