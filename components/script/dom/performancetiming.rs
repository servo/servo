/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceTimingBinding;
use dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::Window;

#[dom_struct]
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
    pub fn new(window: &Window,
               navigation_start: u64,
               navigation_start_precise: f64)
               -> Root<PerformanceTiming> {
        let timing = PerformanceTiming::new_inherited(navigation_start,
                                                      navigation_start_precise);
        reflect_dom_object(box timing, GlobalRef::Window(window),
                           PerformanceTimingBinding::Wrap)
    }
}

impl<'a> PerformanceTimingMethods for &'a PerformanceTiming {
    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/
    // NavigationTiming/Overview.html#dom-performancetiming-navigationstart
    fn NavigationStart(self) -> u64 {
        self.navigationStart
    }
}

pub trait PerformanceTimingHelpers {
    fn NavigationStartPrecise(self) -> f64;
}

impl<'a> PerformanceTimingHelpers for &'a PerformanceTiming {
    fn NavigationStartPrecise(self) -> f64 {
        self.navigationStartPrecise
    }
}
