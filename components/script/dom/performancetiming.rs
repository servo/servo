use crate::dom::bindings::codegen::Bindings::PerformanceTimingBinding::PerformanceTimingMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::codegen::Bindings::PerformanceTimingBinding::DomTypeHolder;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use crate::script_runtime::CanGc;
use std::cell::Cell;

#[dom_struct]
pub struct PerformanceTiming {
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

impl PerformanceTiming {
    pub fn new_inherited(
        navigation_start: u64,
        unload_event_start: u64,
        unload_event_end: u64,
        redirect_start: u64,
        redirect_end: u64,
        fetch_start: u64,
        domain_lookup_start: u64,
        domain_lookup_end: u64,
        connect_start: u64,
        connect_end: u64,
        secure_connection_start: u64,
        request_start: u64,
        response_start: u64,
        response_end: u64,
        dom_loading: u64,
        dom_interactive: u64,
        dom_content_loaded_event_start: u64,
        dom_content_loaded_event_end: u64,
        dom_complete: u64,
        load_event_start: u64,
        load_event_end: u64,
    ) -> PerformanceTiming {
        PerformanceTiming {
            reflector_: Reflector::new(),
            navigation_start: Cell::new(navigation_start),
            unload_event_start: Cell::new(unload_event_start),
            unload_event_end: Cell::new(unload_event_end),
            redirect_start: Cell::new(redirect_start),
            redirect_end: Cell::new(redirect_end),
            fetch_start: Cell::new(fetch_start),
            domain_lookup_start: Cell::new(domain_lookup_start),
            domain_lookup_end: Cell::new(domain_lookup_end),
            connect_start: Cell::new(connect_start),
            connect_end: Cell::new(connect_end),
            secure_connection_start: Cell::new(secure_connection_start),
            request_start: Cell::new(request_start),
            response_start: Cell::new(response_start),
            response_end: Cell::new(response_end),
            dom_loading: Cell::new(dom_loading),
            dom_interactive: Cell::new(dom_interactive),
            dom_content_loaded_event_start: Cell::new(dom_content_loaded_event_start),
            dom_content_loaded_event_end: Cell::new(dom_content_loaded_event_end),
            dom_complete: Cell::new(dom_complete),
            load_event_start: Cell::new(load_event_start),
            load_event_end: Cell::new(load_event_end),
        }
    }

    pub fn new(
        global: &GlobalScope,
        navigation_start: u64,
    ) -> DomRoot<PerformanceTiming> {
        reflect_dom_object(
            Box::new(PerformanceTiming::new_inherited(
                navigation_start,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            )),
            global,
            CanGc::note(),
        )
    }

    pub fn update_unload_event_start(&self, value: u64) {
        self.unload_event_start.set(value);
    }

    pub fn update_unload_event_end(&self, value: u64) {
        self.unload_event_end.set(value);
    }

    pub fn update_redirect_start(&self, value: u64) {
        self.redirect_start.set(value);
    }

    pub fn update_redirect_end(&self, value: u64) {
        self.redirect_end.set(value);
    }

    pub fn update_fetch_start(&self, value: u64) {
        self.fetch_start.set(value);
    }

    pub fn update_domain_lookup_start(&self, value: u64) {
        self.domain_lookup_start.set(value);
    }

    pub fn update_domain_lookup_end(&self, value: u64) {
        self.domain_lookup_end.set(value);
    }

    pub fn update_connect_start(&self, value: u64) {
        self.connect_start.set(value);
    }

    pub fn update_connect_end(&self, value: u64) {
        self.connect_end.set(value);
    }
    pub fn update_secure_connection_start(&self, value: u64) {
        self.secure_connection_start.set(value);
    }
    pub fn update_request_start(&self, value: u64) {
        self.request_start.set(value);
    }
    pub fn update_response_start(&self, value: u64) {
        self.response_start.set(value);
    }
    pub fn update_response_end(&self, value: u64) {
        self.response_end.set(value);
    }
    pub fn update_dom_loading(&self, value: u64) {
        self.dom_loading.set(value);
    }
    pub fn update_dom_interactive(&self, value: u64) {
        self.dom_interactive.set(value);
    }
    pub fn update_dom_content_loaded_event_start(&self, value: u64) {
        self.dom_content_loaded_event_start.set(value);
    }
    pub fn update_dom_content_loaded_event_end(&self, value: u64) {
        self.dom_content_loaded_event_end.set(value);
    }
    pub fn update_dom_complete(&self, value: u64) {
        self.dom_complete.set(value);
    }
    pub fn update_load_event_start(&self, value: u64) {
        self.load_event_start.set(value);
    }
    pub fn update_load_event_end(&self, value: u64) {
        self.load_event_end.set(value);
    }
}


impl PerformanceTimingMethods<DomTypeHelper> for PerformanceTiming {
    fn NavigationStart(&self) -> u64 {
        self.navigation_start.get()
    }

    fn UnloadEventStart(&self) -> u64 {
        self.unload_event_start.get()
    }

    fn UnloadEventEnd(&self) -> u64 {
        self.unload_event_end.get()
    }

    fn RedirectStart(&self) -> u64 {
        self.redirect_start.get()
    }

    fn RedirectEnd(&self) -> u64 {
        self.redirect_end.get()
    }

    fn FetchStart(&self) -> u64 {
        self.fetch_start.get()
    }

    fn DomainLookupStart(&self) -> u64 {
        self.domain_lookup_start.get()
    }

    fn DomainLookupEnd(&self) -> u64 {
        self.domain_lookup_end.get()
    }

    fn ConnectStart(&self) -> u64 {
        self.connect_start.get()
    }

    fn ConnectEnd(&self) -> u64 {
        self.connect_end.get()
    }

    fn SecureConnectionStart(&self) -> u64 {
        self.secure_connection_start.get()
    }

    fn RequestStart(&self) -> u64 {
        self.request_start.get()
    }

    fn ResponseStart(&self) -> u64 {
        self.response_start.get()
    }

    fn ResponseEnd(&self) -> u64 {
        self.response_end.get()
    }

    fn DomLoading(&self) -> u64 {
        self.dom_loading.get()
    }

    fn DomInteractive(&self) -> u64 {
        self.dom_interactive.get()
    }

    fn DomContentLoadedEventStart(&self) -> u64 {
        self.dom_content_loaded_event_start.get()
    }

    fn DomContentLoadedEventEnd(&self) -> u64 {
        self.dom_content_loaded_event_end.get()
    }

    fn DomComplete(&self) -> u64 {
        self.dom_complete.get()
    }

    fn LoadEventStart(&self) -> u64 {
        self.load_event_start.get()
    }

    fn LoadEventEnd(&self) -> u64 {
        self.load_event_end.get()
    }
}
