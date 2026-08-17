/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::DomRoot;
use servo_base::cross_process_instant::CrossProcessInstant;

use crate::dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::document::document::NavigationTiming;
use crate::dom::globalscope::GlobalScope;
use crate::dom::window::Window;

#[dom_struct]
pub(crate) struct PerformanceTiming {
    reflector_: Reflector,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-redirectstart>
    redirect_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-redirectend>
    redirect_end: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-fetchstart>
    fetch_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-domainlookupstart>
    domain_lookup_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-domainlookupend>
    domain_lookup_end: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-connectstart>
    connect_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-connectend>
    connect_end: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-secureconnectstart>
    secure_connection_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-requeststart>
    request_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-responsestart>
    response_start: Cell<u64>,
    /// <https://www.w3.org/TR/navigation-timing/#dom-performancetiming-responseend>
    response_end: Cell<u64>,
}

impl PerformanceTiming {
    pub(crate) fn new_inherited() -> PerformanceTiming {
        PerformanceTiming {
            reflector_: Reflector::new(),
            redirect_start: Default::default(),
            redirect_end: Default::default(),
            fetch_start: Default::default(),
            domain_lookup_start: Default::default(),
            domain_lookup_end: Default::default(),
            connect_start: Default::default(),
            connect_end: Default::default(),
            secure_connection_start: Default::default(),
            request_start: Default::default(),
            response_start: Default::default(),
            response_end: Default::default(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<PerformanceTiming> {
        reflect_dom_object_with_cx(Box::new(PerformanceTiming::new_inherited()), global, cx)
    }

    fn instant_to_millis(instant: Option<CrossProcessInstant>) -> u64 {
        // From <https://www.w3.org/TR/navigation-timing/#terminology>:
        // Throughout this work, time is measured in milliseconds since midnight of January 1, 1970 (UTC).
        let instant = instant.unwrap_or(CrossProcessInstant::epoch());
        let epoch = CrossProcessInstant::epoch();
        (instant - epoch).whole_milliseconds() as u64
    }

    fn navigation_timing(&self) -> Option<Rc<NavigationTiming>> {
        self.global()
            .downcast::<Window>()
            .map(|window| window.Document().navigation_timing())
    }
}

impl PerformanceTimingMethods<crate::DomTypeHolder> for PerformanceTiming {
    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-navigationstart>
    fn NavigationStart(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.navigation_start.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-unloadeventstart
    fn UnloadEventStart(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.unload_event_start.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-unloadeventend>
    fn UnloadEventEnd(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.unload_event_end.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-redirectstart>
    fn RedirectStart(&self) -> u64 {
        self.redirect_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-redirectend>
    fn RedirectEnd(&self) -> u64 {
        self.redirect_end.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-fetchstart>
    fn FetchStart(&self) -> u64 {
        self.fetch_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domainlookupstart>
    fn DomainLookupStart(&self) -> u64 {
        self.domain_lookup_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domainlookupend>
    fn DomainLookupEnd(&self) -> u64 {
        self.domain_lookup_end.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-connectstart>
    fn ConnectStart(&self) -> u64 {
        self.connect_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-connectend>
    fn ConnectEnd(&self) -> u64 {
        self.connect_end.get()
    }

    /// <https://w3c.github.io/navigation-timing#dom-performancetiming-secureconnectionstart>
    fn SecureConnectionStart(&self) -> u64 {
        self.secure_connection_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-requeststart>
    fn RequestStart(&self) -> u64 {
        self.request_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-responsestart>
    fn ResponseStart(&self) -> u64 {
        self.response_start.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-responseend>
    fn ResponseEnd(&self) -> u64 {
        self.response_end.get()
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domloading>
    fn DomLoading(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.dom_loading.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-dominteractive>
    fn DomInteractive(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.dom_interactive.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domcontentloadedeventstart>
    fn DomContentLoadedEventStart(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.dom_content_loaded_event_start.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domcontentloadedeventend>
    fn DomContentLoadedEventEnd(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.dom_content_loaded_event_end.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-domcomplete>
    fn DomComplete(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.dom_complete.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-loadeventstart>
    fn LoadEventStart(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.load_event_start.get()),
        )
    }

    /// <https://w3c.github.io/navigation-timing/#dom-performancetiming-loadeventend>
    fn LoadEventEnd(&self) -> u64 {
        Self::instant_to_millis(
            self.navigation_timing()
                .and_then(|timing| timing.load_event_end.get()),
        )
    }
}
