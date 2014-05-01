/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::PerformanceBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::performancetiming::{PerformanceTiming, PerformanceTimingMethods};
use dom::window::Window;

use time;

pub type DOMHighResTimeStamp = f64;

#[deriving(Encodable)]
pub struct Performance {
    pub reflector_: Reflector,
    pub timing: JS<PerformanceTiming>,
}

impl Performance {
    fn new_inherited(window: &JSRef<Window>) -> Performance {
        Performance {
            reflector_: Reflector::new(),
            timing: PerformanceTiming::new(window).root().root_ref().unrooted(),
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<Performance> {
        let performance = Performance::new_inherited(window);
        reflect_dom_object(~performance, window, PerformanceBinding::Wrap)
    }
}

pub trait PerformanceMethods {
    fn Timing(&self) -> Temporary<PerformanceTiming>;
    fn Now(&self) -> DOMHighResTimeStamp;
}

impl<'a> PerformanceMethods for JSRef<'a, Performance> {
    fn Timing(&self) -> Temporary<PerformanceTiming> {
        Temporary::new(self.timing.clone())
    }

    fn Now(&self) -> DOMHighResTimeStamp {
        let navStart = self.timing.root().NavigationStartPrecise() as f64;
        (time::precise_time_s() - navStart) as DOMHighResTimeStamp
    }
}

impl Reflectable for Performance {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
