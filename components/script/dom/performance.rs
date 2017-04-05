/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceBinding;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::performancetiming::PerformanceTiming;
use dom::window::Window;
use dom_struct::dom_struct;
use time;

pub type DOMHighResTimeStamp = Finite<f64>;

#[dom_struct]
pub struct Performance {
    reflector_: Reflector,
    timing: JS<PerformanceTiming>,
}

impl Performance {
    fn new_inherited(window: &Window,
                     navigation_start: u64,
                     navigation_start_precise: f64) -> Performance {
        Performance {
            reflector_: Reflector::new(),
            timing: JS::from_ref(&*PerformanceTiming::new(window,
                                                            navigation_start,
                                                            navigation_start_precise)),
        }
    }

    pub fn new(window: &Window,
               navigation_start: u64,
               navigation_start_precise: f64) -> Root<Performance> {
        reflect_dom_object(box Performance::new_inherited(window,
                                                          navigation_start,
                                                          navigation_start_precise),
                           window,
                           PerformanceBinding::Wrap)
    }
}

impl PerformanceMethods for Performance {
    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#performance-timing-attribute
    fn Timing(&self) -> Root<PerformanceTiming> {
        Root::from_ref(&*self.timing)
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/HighResolutionTime/Overview.html#dom-performance-now
    fn Now(&self) -> DOMHighResTimeStamp {
        let nav_start = self.timing.navigation_start_precise();
        let now = (time::precise_time_ns() as f64 - nav_start) / 1000000 as f64;
        Finite::wrap(now)
    }
}
