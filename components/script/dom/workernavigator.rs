/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerNavigatorBinding;
use dom::bindings::codegen::Bindings::WorkerNavigatorBinding::WorkerNavigatorMethods;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::str::DOMString;
use dom::navigatorinfo;
use dom::permissions::Permissions;
use dom::workerglobalscope::WorkerGlobalScope;
use dom_struct::dom_struct;
use typeholder::TypeHolderTrait;

// https://html.spec.whatwg.org/multipage/#workernavigator
#[dom_struct]
pub struct WorkerNavigator<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    permissions: MutNullableDom<Permissions<TH>>,
}

impl<TH: TypeHolderTrait> WorkerNavigator<TH> {
    fn new_inherited() -> WorkerNavigator<TH> {
        WorkerNavigator {
            reflector_: Reflector::new(),
            permissions: Default::default(),
        }
    }

    pub fn new(global: &WorkerGlobalScope<TH>) -> DomRoot<WorkerNavigator<TH>> {
        reflect_dom_object(Box::new(WorkerNavigator::new_inherited()),
                           global,
                           WorkerNavigatorBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> WorkerNavigatorMethods<TH> for WorkerNavigator<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-navigator-product
    fn Product(&self) -> DOMString {
        navigatorinfo::Product()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-taintenabled
    fn TaintEnabled(&self) -> bool {
        navigatorinfo::TaintEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appname
    fn AppName(&self) -> DOMString {
        navigatorinfo::AppName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appcodename
    fn AppCodeName(&self) -> DOMString {
        navigatorinfo::AppCodeName()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-platform
    fn Platform(&self) -> DOMString {
        navigatorinfo::Platform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-useragent
    fn UserAgent(&self) -> DOMString {
        navigatorinfo::UserAgent()
    }

    // https://html.spec.whatwg.org/multipage/#dom-navigator-appversion
    fn AppVersion(&self) -> DOMString {
        navigatorinfo::AppVersion()
    }

    // https://html.spec.whatwg.org/multipage/#navigatorlanguage
    fn Language(&self) -> DOMString {
        navigatorinfo::Language()
    }

    // https://w3c.github.io/permissions/#navigator-and-workernavigator-extension
    fn Permissions(&self) -> DomRoot<Permissions<TH>> {
        self.permissions.or_init(|| Permissions::new(&self.global()))
    }
}
