/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::NavigationPreloadManagerBinding::NavigationPreloadState;
use crate::dom::bindings::codegen::Bindings::NavigationPreloadManagerBinding::{
    NavigationPreloadManagerMethods, Wrap,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::ByteString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use std::rc::Rc;

impl MallocSizeOf for NavigationPreloadState {
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        self.enabled.size_of(ops) + self.headerValue.size_of(ops)
    }
}

#[dom_struct]
pub struct NavigationPreloadManager {
    reflector_: Reflector,
    state: DomRefCell<NavigationPreloadState>,
    serviceworker_registration: Dom<ServiceWorkerRegistration>,
}

impl NavigationPreloadManager {
    fn new_inherited(registration: &ServiceWorkerRegistration) -> NavigationPreloadManager {
        NavigationPreloadManager {
            reflector_: Reflector::new(),
            state: DomRefCell::new(NavigationPreloadState::empty()),
            serviceworker_registration: Dom::from_ref(registration),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        registration: &ServiceWorkerRegistration,
    ) -> DomRoot<NavigationPreloadManager> {
        let manager = NavigationPreloadManager::new_inherited(&*registration);
        reflect_dom_object(Box::new(manager), global, Wrap)
    }
}

impl NavigationPreloadManagerMethods for NavigationPreloadManager {
    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#navigation-preload-manager-enable
    fn Enable(&self) -> Rc<Promise> {
        let promise = Promise::new(&*self.global());

        match self.serviceworker_registration.active() {
            Some(_) => {
                // 3.
                let mut state = self.state.borrow_mut();
                state.enabled = true;

                // 4.
                promise.resolve_native(&UndefinedValue());
            },
            None => {
                // 2.
                promise.reject_native(&DOMException::new(
                    &self.global(),
                    DOMErrorName::InvalidStateError,
                ));
            },
        };

        promise
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#navigation-preload-manager-disable
    fn Disable(&self) -> Rc<Promise> {
        let promise = Promise::new(&*self.global());

        match self.serviceworker_registration.active() {
            Some(_) => {
                // 3.
                let mut state = self.state.borrow_mut();
                state.enabled = false;

                // 4.
                promise.resolve_native(&UndefinedValue());
            },
            None => {
                // 2.
                promise.reject_native(&DOMException::new(
                    &self.global(),
                    DOMErrorName::InvalidStateError,
                ));
            },
        };

        promise
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#navigation-preload-manager-setheadervalue
    fn SetHeaderValue(&self, value: ByteString) -> Rc<Promise> {
        let promise = Promise::new(&*self.global());

        match self.serviceworker_registration.active() {
            Some(_) => {
                // 3.
                let mut state = self.state.borrow_mut();
                state.headerValue = Some(value);

                // 4.
                promise.resolve_native(&UndefinedValue());
            },
            None => {
                // 2.
                promise.reject_native(&DOMException::new(
                    &self.global(),
                    DOMErrorName::InvalidStateError,
                ));
            },
        };

        promise
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#navigation-preload-manager-getstate
    fn GetState(&self) -> Rc<Promise> {
        let promise = Promise::new(&*self.global());
        // 2.
        let mut state = NavigationPreloadState::empty();
        let current_state = self.state.borrow();

        // 3.
        if current_state.enabled {
            state.enabled = true;
        }

        // 4.
        state.headerValue = current_state.headerValue.clone();

        // 5.
        promise.resolve_native(&state);

        promise
    }
}
