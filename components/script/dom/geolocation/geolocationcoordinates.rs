/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::GeolocationPositionData;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::GeolocationCoordinatesBinding::GeolocationCoordinatesMethods;
use script_bindings::num::Finite;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use script_bindings::root::DomRoot;

use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub struct GeolocationCoordinates {
    reflector_: Reflector,
    accuracy: Finite<f64>,
    latitude: Finite<f64>,
    longitude: Finite<f64>,
    altitude: Option<Finite<f64>>,
    altitude_accuracy: Option<Finite<f64>>,
    heading: Option<Finite<f64>>,
    speed: Option<Finite<f64>>,
}

/// The outer `Option` reports whether the value was representable while the inner one is the
/// attribute's own nullability.
fn optional_finite(value: Option<f64>) -> Option<Option<Finite<f64>>> {
    match value {
        None => Some(None),
        Some(value) => Finite::new(value).map(Some),
    }
}

impl GeolocationCoordinates {
    fn new_inherited(
        accuracy: Finite<f64>,
        latitude: Finite<f64>,
        longitude: Finite<f64>,
        altitude: Option<Finite<f64>>,
        altitude_accuracy: Option<Finite<f64>>,
        heading: Option<Finite<f64>>,
        speed: Option<Finite<f64>>,
    ) -> Self {
        GeolocationCoordinates {
            reflector_: Reflector::new(),
            accuracy,
            latitude,
            longitude,
            altitude,
            altitude_accuracy,
            heading,
            speed,
        }
    }

    /// Step 1 and 2 of "constructing a `GeolocationPosition`"
    /// Returns `None` if the platform reported a value that cannot be represented as an
    /// unrestricted double, in which case the caller should report `POSITION_UNAVAILABLE`.
    ///
    /// [spec]: https://www.w3.org/TR/geolocation/#dfn-a-new-geolocationposition
    pub(crate) fn from_position_data(
        cx: &mut JSContext,
        global: &GlobalScope,
        data: &GeolocationPositionData,
    ) -> Option<DomRoot<Self>> {
        // > "heading": A double? that represents the heading in degrees, or null if not available
        // > or the device is stationary.
        let heading = if data.speed == Some(0.0) {
            None
        } else {
            optional_finite(data.heading)?
        };

        Some(reflect_dom_object(
            cx,
            Box::new(Self::new_inherited(
                Finite::new(data.accuracy)?,
                Finite::new(data.latitude)?,
                Finite::new(data.longitude)?,
                optional_finite(data.altitude)?,
                optional_finite(data.altitude_accuracy)?,
                heading,
                optional_finite(data.speed)?,
            )),
            global,
        ))
    }
}

impl GeolocationCoordinatesMethods<crate::DomTypeHolder> for GeolocationCoordinates {
    fn Accuracy(&self) -> Finite<f64> {
        self.accuracy
    }

    fn Latitude(&self) -> Finite<f64> {
        self.latitude
    }

    fn Longitude(&self) -> Finite<f64> {
        self.longitude
    }

    fn GetAltitude(&self) -> Option<Finite<f64>> {
        self.altitude
    }

    fn GetAltitudeAccuracy(&self) -> Option<Finite<f64>> {
        self.altitude_accuracy
    }

    fn GetHeading(&self) -> Option<Finite<f64>> {
        self.heading
    }

    fn GetSpeed(&self) -> Option<Finite<f64>> {
        self.speed
    }
}
