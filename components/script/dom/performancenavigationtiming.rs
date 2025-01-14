/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding::{
    NavigationTimingType, PerformanceNavigationTimingMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceresourcetiming::{InitiatorType, PerformanceResourceTiming};
use crate::script_runtime::CanGc;

#[dom_struct]
// https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming
/// Only the current document resource is included in the performance timeline;
/// there is only one PerformanceNavigationTiming object in the performance timeline.
pub(crate) struct PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/#PerformanceResourceTiming
    performanceresourcetiming: PerformanceResourceTiming,
    document: Dom<Document>,
    nav_type: NavigationTimingType,
}

impl PerformanceNavigationTiming {
    fn new_inherited(
        navigation_start: CrossProcessInstant,
        document: &Document,
    ) -> PerformanceNavigationTiming {
        PerformanceNavigationTiming {
            performanceresourcetiming: PerformanceResourceTiming::new_inherited(
                document.url(),
                InitiatorType::Navigation,
                None,
                Some(navigation_start),
            ),
            document: Dom::from_ref(document),
            nav_type: NavigationTimingType::Navigate,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        fetch_start: CrossProcessInstant,
        document: &Document,
    ) -> DomRoot<PerformanceNavigationTiming> {
        reflect_dom_object(
            Box::new(PerformanceNavigationTiming::new_inherited(
                fetch_start,
                document,
            )),
            global,
            CanGc::note(),
        )
    }
}

// https://w3c.github.io/navigation-timing/
impl PerformanceNavigationTimingMethods<crate::DomTypeHolder> for PerformanceNavigationTiming {
    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-unloadeventstart
    fn UnloadEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_unload_event_start())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-unloadeventend
    fn UnloadEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_unload_event_end())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-dominteractive
    fn DomInteractive(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_dom_interactive())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcontentloadedeventstart
    fn DomContentLoadedEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_dom_content_loaded_event_start())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcontentloadedeventstart
    fn DomContentLoadedEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_dom_content_loaded_event_end())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcomplete
    fn DomComplete(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_dom_complete())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-loadeventstart
    fn LoadEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_load_event_start())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-loadeventend
    fn LoadEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_load_event_end())
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-type
    fn Type(&self) -> NavigationTimingType {
        self.nav_type
    }

    // https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-redirectcount
    fn RedirectCount(&self) -> u16 {
        self.document.get_redirect_count()
    }

    // check-tidy: no specs after this line
    // Servo-only timing for when top-level content (not iframes) is complete
    fn TopLevelDomComplete(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.get_top_level_dom_complete())
    }
}
