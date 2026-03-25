/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::WakeLockBinding::{
    WakeLockSentinelMethods, WakeLockType,
};
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

/// <https://w3c.github.io/screen-wake-lock/#the-wakelocksentinel-interface>
#[dom_struct]
pub(crate) struct WakeLockSentinel {
    eventtarget: EventTarget,
    released: Cell<bool>,
    type_: Cell<WakeLockType>,
}

impl WakeLockSentinel {
    pub(crate) fn new_inherited(type_: WakeLockType) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            released: Cell::new(false),
            type_: Cell::new(type_),
        }
    }

    pub(crate) fn new(global: &GlobalScope, type_: WakeLockType, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(type_)), global, can_gc)
    }
}

impl WakeLockSentinelMethods<crate::DomTypeHolder> for WakeLockSentinel {
    /// <https://w3c.github.io/screen-wake-lock/#dom-wakelocksentinel-released>
    fn Released(&self) -> bool {
        self.released.get()
    }

    /// <https://w3c.github.io/screen-wake-lock/#dom-wakelocksentinel-type>
    fn Type(&self) -> WakeLockType {
        self.type_.get()
    }
}
