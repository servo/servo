/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::GeolocationPositionBinding::GeolocationPositionMethods;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::geolocationcoordinates::GeolocationCoordinates;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GeolocationPosition {
    reflector_: Reflector,
    coords: DomRoot<GeolocationCoordinates>,
    timestamp: u64,
}

impl GeolocationPosition {
    pub fn new_inherited(coords: DomRoot<GeolocationCoordinates>, timestamp: u64) -> Self {
        GeolocationPosition {
            reflector_: Reflector::new(),
            coords,
            timestamp,
        }
    }

    pub fn new(global: &GlobalScope, coords: DomRoot<GeolocationCoordinates>, timestamp: u64, can_gc: CanGc) -> DomRoot<Self> {
        reflect_dom_object(Box::new(Self::new_inherited(coords, timestamp)), global, can_gc)
    }
}

impl GeolocationPositionMethods<DomTypeHolder> for GeolocationPosition {
    fn Coords(&self) -> DomRoot<GeolocationCoordinates> {
        self.coords.clone()
    }

    fn Timestamp(&self) -> u64 {
        self.timestamp
    }
}
