/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::fmt::{self, Display, Formatter};

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PermissionStatusBinding::{
    PermissionDescriptor, PermissionName, PermissionState, PermissionStatusMethods,
};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

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
        global: &GlobalScope,
        query: &PermissionDescriptor,
        can_gc: CanGc,
    ) -> DomRoot<PermissionStatus> {
        reflect_dom_object(
            Box::new(PermissionStatus::new_inherited(query.name)),
            global,
            can_gc,
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
    // https://w3c.github.io/permissions/#dom-permissionstatus-state
    fn State(&self) -> PermissionState {
        self.state.get()
    }

    // https://w3c.github.io/permissions/#dom-permissionstatus-onchange
    event_handler!(change, GetOnchange, SetOnchange);
}

impl Display for PermissionName {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
