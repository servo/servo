/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::GeolocationPositionBinding::GeolocationPositionMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::{Dom, DomRoot};

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::geolocationcoordinates::GeolocationCoordinates;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GeolocationPosition {
    reflector_: Reflector,
    coords: Dom<GeolocationCoordinates>,
    timestamp: u64,
}

impl GeolocationPosition {
    fn new_inherited(coords: &GeolocationCoordinates, timestamp: u64) -> Self {
        GeolocationPosition {
            reflector_: Reflector::new(),
            coords: Dom::from_ref(coords),
            timestamp,
        }
    }

    #[expect(unused)]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        coords: &GeolocationCoordinates,
        timestamp: u64,
    ) -> DomRoot<Self> {
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(coords, timestamp)), global, cx)
    }
}

impl GeolocationPositionMethods<DomTypeHolder> for GeolocationPosition {
    fn Coords(&self) -> DomRoot<GeolocationCoordinates> {
        DomRoot::from_ref(&*self.coords.clone())
    }

    fn Timestamp(&self) -> u64 {
        self.timestamp
    }
}
