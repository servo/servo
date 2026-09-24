/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::GeolocationPositionData;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::GeolocationPositionBinding::GeolocationPositionMethods;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use script_bindings::root::{Dom, DomRoot};

use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::geolocationcoordinates::GeolocationCoordinates;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GeolocationPosition {
    reflector_: Reflector,
    coords: Dom<GeolocationCoordinates>,
    timestamp: u64,
    /// <https://www.w3.org/TR/geolocation/#dfn-ishighaccuracy>
    is_high_accuracy: bool,
}

impl GeolocationPosition {
    fn new_inherited(
        coords: &GeolocationCoordinates,
        timestamp: u64,
        is_high_accuracy: bool,
    ) -> Self {
        GeolocationPosition {
            reflector_: Reflector::new(),
            coords: Dom::from_ref(coords),
            timestamp,
            is_high_accuracy,
        }
    }

    /// <https://www.w3.org/TR/geolocation/#dfn-a-new-geolocationposition>
    ///
    /// Returns `None` if `data` contains a value that cannot be represented as an unrestricted
    /// double, in which case the caller should report `POSITION_UNAVAILABLE`.
    pub(crate) fn from_position_data(
        cx: &mut JSContext,
        global: &GlobalScope,
        data: &GeolocationPositionData,
        timestamp: u64,
        is_high_accuracy: bool,
    ) -> Option<DomRoot<Self>> {
        // Step 1 and 2. Create the coordinates from positionData.
        let coords = GeolocationCoordinates::from_position_data(cx, global, data)?;
        // Step 3. Return a newly created GeolocationPosition.
        Some(reflect_dom_object(
            cx,
            Box::new(Self::new_inherited(&coords, timestamp, is_high_accuracy)),
            global,
        ))
    }

    /// <https://www.w3.org/TR/geolocation/#dfn-ishighaccuracy>
    pub(crate) fn is_high_accuracy(&self) -> bool {
        self.is_high_accuracy
    }

    /// <https://www.w3.org/TR/geolocation/#dom-geolocationposition-timestamp>
    pub(crate) fn timestamp(&self) -> u64 {
        self.timestamp
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
