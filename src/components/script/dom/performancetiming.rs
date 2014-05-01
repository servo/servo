/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::PerformanceTimingBinding;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[deriving(Encodable)]
pub struct PerformanceTiming {
    pub reflector_: Reflector,
    pub navigationStart: u64,
    pub navigationStartPrecise: f64,
}

impl PerformanceTiming {
    pub fn new_inherited(navStart: u64, navStartPrecise: f64)
                         -> PerformanceTiming {
        PerformanceTiming {
            reflector_: Reflector::new(),
            navigationStart: navStart,
            navigationStartPrecise: navStartPrecise,
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<PerformanceTiming> {
        let timing = PerformanceTiming::new_inherited(window.navigationStart,
                                                      window.navigationStartPrecise);
        reflect_dom_object(~timing, window, PerformanceTimingBinding::Wrap)
    }
}

pub trait PerformanceTimingMethods {
    fn NavigationStart(&self) -> u64;
    fn NavigationStartPrecise(&self) -> f64;
}

impl<'a> PerformanceTimingMethods for JSRef<'a, PerformanceTiming> {
    fn NavigationStart(&self) -> u64 {
        self.navigationStart
    }

    fn NavigationStartPrecise(&self) -> f64 {
        self.navigationStartPrecise
    }
}

impl Reflectable for PerformanceTiming {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
