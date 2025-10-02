/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dom_struct::dom_struct;
use rustc_hash::FxHashSet;
use script_bindings::codegen::GenericBindings::GeolocationBinding::Geolocation_Binding::GeolocationMethods;
use script_bindings::codegen::GenericBindings::GeolocationBinding::{
    PositionCallback, PositionOptions,
};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct Geolocation {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/geolocation/#dfn-watchids>
    watch_ids: RefCell<FxHashSet<u32>>,
    next_watch_id: Cell<u32>,
}

impl Geolocation {
    fn new_inherited() -> Self {
        Geolocation {
            reflector_: Reflector::new(),
            watch_ids: RefCell::new(FxHashSet::default()),
            next_watch_id: Cell::new(1),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited()), global, can_gc)
    }
}

impl GeolocationMethods<DomTypeHolder> for Geolocation {
    /// <https://www.w3.org/TR/geolocation/#dom-geolocation-getcurrentposition>
    fn GetCurrentPosition(
        &self,
        _success_callback: Rc<PositionCallback<DomTypeHolder>>,
        _options: &PositionOptions,
    ) {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        // if !self.global().as_window().Document().is_active() {
        // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
        // Step 1.2 Terminate this algorithm.
        // return;
        // }
        // Step 2. Request a position passing this, successCallback, errorCallback, and options.
        // FIXME(arihant2math)
    }

    /// <https://www.w3.org/TR/geolocation/#watchposition-method>
    fn WatchPosition(
        &self,
        _success_callback: Rc<PositionCallback<DomTypeHolder>>,
        _options: &PositionOptions,
    ) -> i32 {
        // Step 1. If this's relevant global object's associated Document is not fully active:
        if !self.global().as_window().Document().is_active() {
            // Step 1.1 Call back with error errorCallback and POSITION_UNAVAILABLE.
            // Step 1.2 Return 0.
            return 0;
        }
        // Step 2. Let watchId be an implementation-defined unsigned long that is greater than zero.
        let watch_id = self.next_watch_id.get();
        self.next_watch_id.set(watch_id + 1);
        // Step 3. Append watchId to this's [[watchIDs]].
        self.watch_ids.borrow_mut().insert(watch_id);
        // Step 4. Request a position passing this, successCallback, errorCallback, options, and watchId.
        // FIXME(arihant2math)
        // Step 5. Return watchId.
        watch_id as i32
    }

    /// <https://www.w3.org/TR/geolocation/#clearwatch-method>
    fn ClearWatch(&self, watch_id: i32) {
        let watch_id = u32::try_from(watch_id).ok();
        if let Some(id) = watch_id {
            self.watch_ids.borrow_mut().remove(&id);
        }
    }
}
