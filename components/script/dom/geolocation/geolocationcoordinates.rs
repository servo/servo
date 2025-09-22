/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::GeolocationCoordinatesBinding::GeolocationCoordinatesMethods;
use script_bindings::num::Finite;
use script_bindings::reflector::Reflector;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use crate::dom::bindings::reflector::reflect_dom_object;
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

impl GeolocationCoordinates {
    pub fn new_inherited(accuracy: f64, latitude: f64, longitude: f64, altitude: Option<f64>, altitude_accuracy: Option<f64>, heading: Option<f64>, speed: Option<f64>) -> Option<Self> {
        Some(GeolocationCoordinates {
            reflector_: Reflector::new(),
            accuracy: Finite::new(accuracy)?,
            latitude: Finite::new(latitude)?,
            longitude: Finite::new(longitude)?,
            altitude: altitude.map(|a| Finite::new(a)).flatten(),
            altitude_accuracy: altitude_accuracy.map(|a| Finite::new(a)).flatten(),
            heading: heading.map(|h| Finite::new(h)).flatten(),
            speed: speed.map(|s| Finite::new(s)).flatten(),
        })
    }

    pub fn new(global: &GlobalScope, accuracy: f64, latitude: f64, longitude: f64, altitude: Option<f64>, altitude_accuracy: Option<f64>, heading: Option<f64>, speed: Option<f64>, can_gc: CanGc) -> Option<DomRoot<Self>> {
        Some(reflect_dom_object(Box::new(Self::new_inherited(accuracy, latitude, longitude, altitude, altitude_accuracy, heading, speed)?), global, can_gc))
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
