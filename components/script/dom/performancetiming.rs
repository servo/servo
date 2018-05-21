/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceTimingBinding;
use dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::document::Document;
use dom::window::Window;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PerformanceTiming {
    reflector_: Reflector,
    navigation_start: u64,
    navigation_start_precise: u64,
    document: Dom<Document>,
}

impl PerformanceTiming {
    fn new_inherited(nav_start: u64,
                     nav_start_precise: u64,
                     document: &Document)
                         -> PerformanceTiming {
        PerformanceTiming {
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
               -> DomRoot<PerformanceTiming> {
        let timing = PerformanceTiming::new_inherited(navigation_start,
                                                      navigation_start_precise,
                                                      &window.Document());
        reflect_dom_object(Box::new(timing),
                           window,
                           PerformanceTimingBinding::Wrap)
    }
}

impl PerformanceTimingMethods for PerformanceTiming {
    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-navigationStart
    fn NavigationStart(&self) -> u64 {
        self.navigation_start
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domLoading
    fn DomLoading(&self) -> u64 {
        self.document.get_dom_loading()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domInteractive
    fn DomInteractive(&self) -> u64 {
        self.document.get_dom_interactive()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domContentLoadedEventStart
    fn DomContentLoadedEventStart(&self) -> u64 {
        self.document.get_dom_content_loaded_event_start()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domContentLoadedEventEnd
    fn DomContentLoadedEventEnd(&self) -> u64 {
        self.document.get_dom_content_loaded_event_end()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-domComplete
    fn DomComplete(&self) -> u64 {
        self.document.get_dom_complete()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-loadEventStart
    fn LoadEventStart(&self) -> u64 {
        self.document.get_load_event_start()
    }

    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-loadEventEnd
    fn LoadEventEnd(&self) -> u64 {
        self.document.get_load_event_end()
    }

    // check-tidy: no specs after this line
    // Servo-only timing for when top-level content (not iframes) is complete
    fn TopLevelDomComplete(&self) -> u64 {
        self.document.get_top_level_dom_complete()
    }
}


impl PerformanceTiming {
    pub fn navigation_start_precise(&self) -> u64 {
        self.navigation_start_precise
    }
}
