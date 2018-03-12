/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![allow(unused)]

use dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding::NavigationType;
use dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding::PerformanceNavigationTimingMethods;
use dom::bindings::num::Finite;
use dom::bindings::root::Dom;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::performanceresourcetiming::PerformanceResourceTiming;
use dom_struct::dom_struct;


#[dom_struct]
// https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming
pub struct PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/#PerformanceResourceTiming
    performanceresourcetiming: PerformanceResourceTiming,
    navigation_start: u64,
    navigation_start_precise: u64,
    document: Dom<Document>,
    nav_type: NavigationType,
    redirect_count: u32,
}

impl PerformanceNavigationTiming {
    fn new_inherited(nav_start: u64,
                     nav_start_precise: u64,
                     document: &Document)
                         -> PerformanceNavigationTiming {
        PerformanceNavigationTiming {
            performanceresourcetiming: PerformanceResourceTiming::new_inherited(
                document.url(),
                DOMString::from("navigation"),
                None,
                nav_start_precise as f64),
            navigation_start: nav_start,
            navigation_start_precise: nav_start_precise,
            document: Dom::from_ref(document),
            nav_type: NavigationType::Navigate,
            redirect_count: 0,
        }
    }

    pub fn set_type(&mut self, nav_type: NavigationType) {
        self.nav_type = nav_type
    }
}

// https://w3c.github.io/navigation-timing/
impl PerformanceNavigationTimingMethods for PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/
    fn UnloadEventStart(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_unload_event_start() as f64)
    }

    // https://w3c.github.io/navigation-timing/
    fn UnloadEventEnd(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.document.get_unload_event_end() as f64)
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

    // TODO type
    fn Type(&self) -> NavigationType {
        self.nav_type.clone()
    }

    fn RedirectCount(&self) -> u16 {
        self.document.get_redirect_count()
    }
}

impl PerformanceNavigationTiming {
    pub fn navigation_start_precise(&self) -> u64 {
        self.navigation_start_precise
    }
}
