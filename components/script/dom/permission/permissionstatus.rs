/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName, PermissionState, PermissionStatusMethods,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;

// https://w3c.github.io/permissions/#permissionstatus
#[dom_struct]
pub(crate) struct PermissionStatus {
    eventtarget: EventTarget,
    state: Cell<PermissionState>,
    query: Cell<PermissionName>,
}

impl PermissionStatus {
    pub(crate) fn new_inherited(query: PermissionName) -> PermissionStatus {
        PermissionStatus {
            eventtarget: EventTarget::new_inherited(),
            state: Cell::new(PermissionState::Denied),
            query: Cell::new(query),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        query: &PermissionDescriptor,
    ) -> DomRoot<PermissionStatus> {
        reflect_dom_object_with_cx(
            Box::new(PermissionStatus::new_inherited(query.name)),
            global,
            cx,
        )
    }

    pub(crate) fn set_state(&self, state: PermissionState) {
        self.state.set(state);
    }

    pub(crate) fn get_query(&self) -> PermissionName {
        self.query.get()
    }
}

impl PermissionStatusMethods<crate::DomTypeHolder> for PermissionStatus {
    /// <https://w3c.github.io/permissions/#dom-permissionstatus-state>
    fn State(&self) -> PermissionState {
        self.state.get()
    }

    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}
