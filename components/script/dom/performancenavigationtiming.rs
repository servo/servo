/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding;
use dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding::PerformanceNavigationTimingMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::document::Document;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming
    reflector_: Reflector,
    navigation_start: u64,
    navigation_start_precise: u64,
    document: Dom<Document>,
}

impl PerformanceNavigationTiming {
    fn new_inherited(nav_start: u64,
                     nav_start_precise: u64,
                     document: &Document)
                         -> PerformanceNavigationTiming {
        PerformanceNavigationTiming {
            reflector_: Reflector::new(),
            navigation_start: nav_start,
            navigation_start_precise: nav_start_precise,
            document: Dom::from_ref(document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window,
               navigation_start: u64,
               navigation_start_precise: u64)
               -> DomRoot<PerformanceNavigationTiming> {
        let timing = PerformanceNavigationTiming::new_inherited(navigation_start,
                                                      navigation_start_precise,
                                                      &window.Document());
        reflect_dom_object(Box::new(timing),
                           window,
                           PerformanceNavigationTimingBinding::Wrap)
    }
}

// https://w3c.github.io/navigation-timing/
impl PerformanceNavigationTimingMethods for PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/
    fn UnloadEventStart(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/navigation-timing/
    fn UnloadEventEnd(&self) -> DOMHighResTimeStamp {
        // TODO
        Finite::wrap(Default::default())
    }

    // https://w3c.github.io/navigation-timing/
    fn DomInteractive(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_dom_interactive() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn DomContentLoadedEventStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_dom_content_loaded_event_start() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn DomContentLoadedEventEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_dom_content_loaded_event_end() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn DomComplete(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_dom_complete() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn LoadEventStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_load_event_start() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn LoadEventEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_load_event_end() as f64)
    }

    // check-tidy: no specs after this line
    // Servo-only timing for when top-level content (not iframes) is complete
    fn TopLevelDomComplete(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_top_level_dom_complete() as f64)
    }
}


impl PerformanceNavigationTiming {
    pub fn navigation_start_precise(&self) -> u64 {
        self.navigation_start_precise
    }
}
