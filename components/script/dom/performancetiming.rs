/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceTimingBinding;
use dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use dom::bindings::global::Window;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[jstraceable]
#[must_root]
pub struct PerformanceTiming {
    reflector_: Reflector,
    navigationStart: u64,
    navigationStartPrecise: f64,
}

impl PerformanceTiming {
    fn new_inherited(navStart: u64, navStartPrecise: f64)
                         -> PerformanceTiming {
        PerformanceTiming {
            reflector_: Reflector::new(),
            navigationStart: navStart,
            navigationStartPrecise: navStartPrecise,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: JSRef<Window>) -> Temporary<PerformanceTiming> {
        let timing = PerformanceTiming::new_inherited(window.navigationStart,
                                                      window.navigationStartPrecise);
        reflect_dom_object(box timing, &Window(window),
                           PerformanceTimingBinding::Wrap)
    }
}

impl<'a> PerformanceTimingMethods for JSRef<'a, PerformanceTiming> {
    fn NavigationStart(self) -> u64 {
        self.navigationStart
    }
}

pub trait PerformanceTimingHelpers {
    fn NavigationStartPrecise(self) -> f64;
}

impl<'a> PerformanceTimingHelpers for JSRef<'a, PerformanceTiming> {
    fn NavigationStartPrecise(self) -> f64 {
        self.navigationStartPrecise
    }
}

impl Reflectable for PerformanceTiming {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
