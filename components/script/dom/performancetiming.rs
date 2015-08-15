/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceTimingBinding;
use dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::document::Document;
use dom::window::Window;

#[dom_struct]
pub struct PerformanceTiming {
    reflector_: Reflector,
    navigationStart: u64,
    navigationStartPrecise: f64,
    document: JS<Document>,
}

impl PerformanceTiming {
    fn new_inherited(navStart: u64,
                     navStartPrecise: f64,
                     document: &Document)
                         -> PerformanceTiming {
        PerformanceTiming {
            reflector_: Reflector::new(),
            navigationStart: navStart,
            navigationStartPrecise: navStartPrecise,
            document: JS::from_ref(document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window,
               navigation_start: u64,
               navigation_start_precise: f64)
               -> Root<PerformanceTiming> {

        let timing = PerformanceTiming::new_inherited(navigation_start,
                                                      navigation_start_precise,
                                                      window.Document().r());
        reflect_dom_object(box timing, GlobalRef::Window(window),
                           PerformanceTimingBinding::Wrap)
    }
}

impl PerformanceTimingMethods for PerformanceTiming {
    // https://w3c.github.io/navigation-timing/#widl-PerformanceTiming-navigationStart
    fn NavigationStart(&self) -> u64 {
        self.navigationStart
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
}


impl PerformanceTiming {
    pub fn NavigationStartPrecise(&self) -> f64 {
        self.navigationStartPrecise
    }
}
