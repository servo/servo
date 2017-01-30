/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use core::clone::Clone;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{self, PermissionDescriptor, PermissionName};
use dom::bindings::codegen::Bindings::PermissionStatusBinding::{PermissionState, PermissionStatusMethods};
use dom::bindings::js::Root;
use dom::bindings::reflector::reflect_dom_object;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;

// https://w3c.github.io/permissions/#permissionstatus
#[dom_struct]
pub struct PermissionStatus {
    eventtarget: EventTarget,
    state: DOMRefCell<PermissionState>,
    query: DOMRefCell<PermissionName>,
}

impl PermissionStatus {
    pub fn new_inherited(query: PermissionName) -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            state: DOMRefCell::new(PermissionState::Denied),
            query: DOMRefCell::new(query),
        }
    }

    pub fn new(global: &GlobalScope, query: &PermissionDescriptor) -> Root<PermissionStatus> {
        reflect_dom_object(box PermissionStatus::new_inherited(query.name),
                           global,
                           PermissionStatusBinding::Wrap)
    }

    pub fn set_state(&self, state: PermissionState) {
        *self.state.borrow_mut() = state;
    }

    pub fn get_query(&self) -> PermissionName {
        self.query.borrow().clone()
    }
}

impl PermissionStatusMethods for PermissionStatus {
    // https://w3c.github.io/permissions/#dom-permissionstatus-state
    fn State(&self) -> PermissionState {
        self.state.borrow().clone()
    }

    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
    event_handler!(onchange, GetOnchange, SetOnchange);
}
