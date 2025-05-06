/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PerformanceTiming {
    reflector_: Reflector,
    navigation_start: Cell<u64>,
    unload_event_start: Cell<u64>,
    unload_event_end: Cell<u64>,
    redirect_start: Cell<u64>,
    redirect_end: Cell<u64>,
    fetch_start: Cell<u64>,
    domain_lookup_start: Cell<u64>,
    domain_lookup_end: Cell<u64>,
    connect_start: Cell<u64>,
    connect_end: Cell<u64>,
    secure_connection_start: Cell<u64>,
    request_start: Cell<u64>,
    response_start: Cell<u64>,
    response_end: Cell<u64>,
    dom_loading: Cell<u64>,
    dom_interactive: Cell<u64>,
    dom_content_loaded_event_start: Cell<u64>,
    dom_content_loaded_event_end: Cell<u64>,
    dom_complete: Cell<u64>,
    load_event_start: Cell<u64>,
    load_event_end: Cell<u64>,
}

#[allow(dead_code)]
impl PerformanceTiming {
    pub fn new(
        global: &GlobalScope,
        navigation_start: u64,
        can_gc: CanGc,
    ) -> DomRoot<PerformanceTiming> {
        reflect_dom_object(
            Box::new(PerformanceTiming {
                reflector_: Reflector::new(),
                navigation_start: Cell::new(navigation_start),
                unload_event_start: Cell::new(0),
                unload_event_end: Cell::new(0),
                redirect_start: Cell::new(0),
                redirect_end: Cell::new(0),
                fetch_start: Cell::new(0),
                domain_lookup_start: Cell::new(0),
                domain_lookup_end: Cell::new(0),
                connect_start: Cell::new(0),
                connect_end: Cell::new(0),
                secure_connection_start: Cell::new(0),
                request_start: Cell::new(0),
                response_start: Cell::new(0),
                response_end: Cell::new(0),
                dom_loading: Cell::new(0),
                dom_interactive: Cell::new(0),
                dom_content_loaded_event_start: Cell::new(0),
                dom_content_loaded_event_end: Cell::new(0),
                dom_complete: Cell::new(0),
                load_event_start: Cell::new(0),
                load_event_end: Cell::new(0),
            }),
            global,
            can_gc,
        )
    }

    pub(crate) fn update_unload_event_start(&self, value: u64) {
        self.unload_event_start.set(value);
    }

    pub(crate) fn update_unload_event_end(&self, value: u64) {
        self.unload_event_end.set(value);
    }

    pub(crate) fn update_redirect_start(&self, value: u64) {
        self.redirect_start.set(value);
    }

    pub(crate) fn update_redirect_end(&self, value: u64) {
        self.redirect_end.set(value);
    }

    pub(crate) fn update_fetch_start(&self, value: u64) {
        self.fetch_start.set(value);
    }

    pub(crate) fn update_domain_lookup_start(&self, value: u64) {
        self.domain_lookup_start.set(value);
    }

    pub(crate) fn update_domain_lookup_end(&self, value: u64) {
        self.domain_lookup_end.set(value);
    }

    pub(crate) fn update_connect_start(&self, value: u64) {
        self.connect_start.set(value);
    }

    pub(crate) fn update_connect_end(&self, value: u64) {
        self.connect_end.set(value);
    }
    pub(crate) fn update_secure_connection_start(&self, value: u64) {
        self.secure_connection_start.set(value);
    }
    pub(crate) fn update_request_start(&self, value: u64) {
        self.request_start.set(value);
    }
    pub(crate) fn update_response_start(&self, value: u64) {
        self.response_start.set(value);
    }
    pub(crate) fn update_response_end(&self, value: u64) {
        self.response_end.set(value);
    }
    pub(crate) fn update_dom_loading(&self, value: u64) {
        self.dom_loading.set(value);
    }
    pub(crate) fn update_dom_interactive(&self, value: u64) {
        self.dom_interactive.set(value);
    }
    pub(crate) fn update_dom_content_loaded_event_start(&self, value: u64) {
        self.dom_content_loaded_event_start.set(value);
    }
    pub(crate) fn update_dom_content_loaded_event_end(&self, value: u64) {
        self.dom_content_loaded_event_end.set(value);
    }
    pub(crate) fn update_dom_complete(&self, value: u64) {
        self.dom_complete.set(value);
    }
    pub(crate) fn update_load_event_start(&self, value: u64) {
        self.load_event_start.set(value);
    }
    pub(crate) fn update_load_event_end(&self, value: u64) {
        self.load_event_end.set(value);
    }
}

impl PerformanceTimingMethods<crate::DomTypeHolder> for PerformanceTiming {
    // https://w3c.github.io/navigation-timing/#dom-performancetiming-navigationstart
    fn NavigationStart(&self) -> u64 {
        self.navigation_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-unloadeventstart
    fn UnloadEventStart(&self) -> u64 {
        self.unload_event_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-unloadeventend
    fn UnloadEventEnd(&self) -> u64 {
        self.unload_event_end.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-redirectstart
    fn RedirectStart(&self) -> u64 {
        self.redirect_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-redirectend
    fn RedirectEnd(&self) -> u64 {
        self.redirect_end.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-fetchstart
    fn FetchStart(&self) -> u64 {
        self.fetch_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domainlookupstart
    fn DomainLookupStart(&self) -> u64 {
        self.domain_lookup_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domainlookupend
    fn DomainLookupEnd(&self) -> u64 {
        self.domain_lookup_end.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-connectstart
    fn ConnectStart(&self) -> u64 {
        self.connect_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-connectend
    fn ConnectEnd(&self) -> u64 {
        self.connect_end.get()
    }

    // https://w3c.github.io/navigation-timing#dom-performancetiming-secureconnectionstart
    fn SecureConnectionStart(&self) -> u64 {
        self.secure_connection_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-requeststart
    fn RequestStart(&self) -> u64 {
        self.request_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-responsestart
    fn ResponseStart(&self) -> u64 {
        self.response_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-responseend
    fn ResponseEnd(&self) -> u64 {
        self.response_end.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domloading
    fn DomLoading(&self) -> u64 {
        self.dom_loading.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-dominteractive
    fn DomInteractive(&self) -> u64 {
        self.dom_interactive.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domcontentloadedeventstart
    fn DomContentLoadedEventStart(&self) -> u64 {
        self.dom_content_loaded_event_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domcontentloadedeventend
    fn DomContentLoadedEventEnd(&self) -> u64 {
        self.dom_content_loaded_event_end.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-domcomplete
    fn DomComplete(&self) -> u64 {
        self.dom_complete.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-loadeventstart
    fn LoadEventStart(&self) -> u64 {
        self.load_event_start.get()
    }

    // https://w3c.github.io/navigation-timing/#dom-performancetiming-loadeventend
    fn LoadEventEnd(&self) -> u64 {
        self.load_event_end.get()
    }
}
