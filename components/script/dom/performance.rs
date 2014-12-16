/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceBinding;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::performancetiming::{PerformanceTiming, PerformanceTimingHelpers};
use dom::window::Window;
use time;

pub type DOMHighResTimeStamp = f64;

#[dom_struct]
pub struct Performance {
    reflector_: Reflector,
    timing: JS<PerformanceTiming>,
}

impl Performance {
    fn new_inherited(window: JSRef<Window>,
                     navigation_start: u64,
                     navigation_start_precise: f64) -> Performance {
        Performance {
            reflector_: Reflector::new(),
            timing: JS::from_rooted(PerformanceTiming::new(window,
                                                           navigation_start,
                                                           navigation_start_precise)),
        }
    }

    pub fn new(window: JSRef<Window>,
               navigation_start: u64,
               navigation_start_precise: f64) -> Temporary<Performance> {
        reflect_dom_object(box Performance::new_inherited(window,
                                                          navigation_start,
                                                          navigation_start_precise),
                           global::Window(window),
                           PerformanceBinding::Wrap)
    }
}

impl<'a> PerformanceMethods for JSRef<'a, Performance> {
    fn Timing(self) -> Temporary<PerformanceTiming> {
        Temporary::new(self.timing.clone())
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/HighResolutionTime/Overview.html#dom-performance-now
    fn Now(self) -> DOMHighResTimeStamp {
        let navStart = self.timing.root().NavigationStartPrecise();
        (time::precise_time_ns() as f64 - navStart) * 1000000u as DOMHighResTimeStamp
    }
}

impl Reflectable for Performance {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
