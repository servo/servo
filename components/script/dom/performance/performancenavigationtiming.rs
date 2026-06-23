/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use script_bindings::reflector::reflect_dom_object_with_cx;

use super::performanceresourcetiming::{InitiatorType, PerformanceResourceTiming};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceNavigationTimingBinding::{
    NavigationTimingType, PerformanceNavigationTimingMethods,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::document::Document;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
/// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming>
/// Only the current document resource is included in the performance timeline;
/// there is only one PerformanceNavigationTiming object in the performance timeline.
pub(crate) struct PerformanceNavigationTiming {
    /// <https://w3c.github.io/navigation-timing/#PerformanceResourceTiming>
    performanceresourcetiming: PerformanceResourceTiming,
    document: Dom<Document>,
    nav_type: NavigationTimingType,
}

impl PerformanceNavigationTiming {
    fn new_inherited(document: &Document) -> PerformanceNavigationTiming {
        PerformanceNavigationTiming {
            performanceresourcetiming: PerformanceResourceTiming::new_inherited(
                document.url(),
                InitiatorType::Navigation,
                document
                    .resource_fetch_timing()
                    .as_ref()
                    .unwrap_or(&ResourceFetchTiming::new(ResourceTimingType::None)),
            ),
            document: Dom::from_ref(document),
            nav_type: NavigationTimingType::Navigate,
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        document: &Document,
    ) -> DomRoot<PerformanceNavigationTiming> {
        reflect_dom_object_with_cx(
            Box::new(PerformanceNavigationTiming::new_inherited(document)),
            global,
            cx,
        )
    }
}

// https://w3c.github.io/navigation-timing/
impl PerformanceNavigationTimingMethods<crate::DomTypeHolder> for PerformanceNavigationTiming {
    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-unloadeventstart>
    fn UnloadEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().unload_event_start.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-unloadeventend>
    fn UnloadEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().unload_event_end.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-dominteractive>
    fn DomInteractive(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().dom_interactive.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcontentloadedeventstart>
    fn DomContentLoadedEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(
                self.document
                    .navigation_timing()
                    .dom_content_loaded_event_start
                    .get(),
            )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcontentloadedeventend>
    fn DomContentLoadedEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(
                self.document
                    .navigation_timing()
                    .dom_content_loaded_event_end
                    .get(),
            )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-domcomplete>
    fn DomComplete(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().dom_complete.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-loadeventstart>
    fn LoadEventStart(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().load_event_start.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-loadeventend>
    fn LoadEventEnd(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(self.document.navigation_timing().load_event_end.get())
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-type>
    fn Type(&self) -> NavigationTimingType {
        self.nav_type
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancenavigationtiming-redirectcount>
    fn RedirectCount(&self) -> u16 {
        self.document.get_redirect_count()
    }

    // check-tidy: no specs after this line
    // Servo-only timing for when top-level content (not iframes) is complete
    fn TopLevelDomComplete(&self) -> DOMHighResTimeStamp {
        self.upcast::<PerformanceResourceTiming>()
            .to_dom_high_res_time_stamp(
                self.document
                    .navigation_timing()
                    .top_level_dom_complete
                    .get(),
            )
    }
}
