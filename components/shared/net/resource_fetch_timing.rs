/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use servo_arc::Arc;
use servo_base::cross_process_instant::CrossProcessInstant;

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ResourceFetchTiming {
    pub domain_lookup_start: Option<CrossProcessInstant>,
    pub timing_check_passed: bool,
    pub timing_type: ResourceTimingType,
    /// Number of redirects until final resource (currently limited to 20)
    pub redirect_count: u16,
    pub request_start: Option<CrossProcessInstant>,
    pub secure_connection_start: Option<CrossProcessInstant>,
    pub response_start: Option<CrossProcessInstant>,
    pub fetch_start: Option<CrossProcessInstant>,
    pub response_end: Option<CrossProcessInstant>,
    pub redirect_start: Option<CrossProcessInstant>,
    pub redirect_end: Option<CrossProcessInstant>,
    pub connect_start: Option<CrossProcessInstant>,
    pub connect_end: Option<CrossProcessInstant>,
    pub start_time: Option<CrossProcessInstant>,
    pub preloaded: bool,
}

#[derive(Clone)]
pub enum RedirectStartValue {
    Zero,
    FetchStart,
}

#[derive(Clone)]
pub enum RedirectEndValue {
    Zero,
    ResponseEnd,
}

// TODO: refactor existing code to use this enum for setting time attributes
// suggest using this with all time attributes in the future
#[derive(Clone)]
pub enum ResourceTimeValue {
    Zero,
    Now,
    FetchStart,
    RedirectStart,
}

#[derive(Clone)]
pub enum ResourceAttribute {
    RedirectCount(u16),
    DomainLookupStart,
    RequestStart,
    ResponseStart,
    RedirectStart(RedirectStartValue),
    RedirectEnd(RedirectEndValue),
    FetchStart,
    ConnectStart(CrossProcessInstant),
    ConnectEnd(CrossProcessInstant),
    SecureConnectionStart,
    ResponseEnd,
    StartTime(ResourceTimeValue),
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum ResourceTimingType {
    Resource,
    Navigation,
    Error,
    None,
}

impl ResourceFetchTiming {
    pub fn new(timing_type: ResourceTimingType) -> ResourceFetchTiming {
        ResourceFetchTiming {
            timing_type,
            timing_check_passed: true,
            domain_lookup_start: None,
            redirect_count: 0,
            secure_connection_start: None,
            request_start: None,
            response_start: None,
            fetch_start: None,
            redirect_start: None,
            redirect_end: None,
            connect_start: None,
            connect_end: None,
            response_end: None,
            start_time: None,
            preloaded: false,
        }
    }

    // TODO currently this is being set with precise time ns when it should be time since
    // time origin (as described in Performance::now)
    pub fn set_attribute(&mut self, attribute: ResourceAttribute) {
        let should_attribute_always_be_updated = matches!(
            attribute,
            ResourceAttribute::FetchStart |
                ResourceAttribute::ResponseEnd |
                ResourceAttribute::StartTime(_)
        );
        if !self.timing_check_passed && !should_attribute_always_be_updated {
            return;
        }
        let now = Some(CrossProcessInstant::now());
        match attribute {
            ResourceAttribute::DomainLookupStart => self.domain_lookup_start = now,
            ResourceAttribute::RedirectCount(count) => self.redirect_count = count,
            ResourceAttribute::RequestStart => self.request_start = now,
            ResourceAttribute::ResponseStart => self.response_start = now,
            ResourceAttribute::RedirectStart(val) => match val {
                RedirectStartValue::Zero => self.redirect_start = None,
                RedirectStartValue::FetchStart => {
                    if self.redirect_start.is_none() {
                        self.redirect_start = self.fetch_start
                    }
                },
            },
            ResourceAttribute::RedirectEnd(val) => match val {
                RedirectEndValue::Zero => self.redirect_end = None,
                RedirectEndValue::ResponseEnd => self.redirect_end = self.response_end,
            },
            ResourceAttribute::FetchStart => self.fetch_start = now,
            ResourceAttribute::ConnectStart(instant) => self.connect_start = Some(instant),
            ResourceAttribute::ConnectEnd(instant) => self.connect_end = Some(instant),
            ResourceAttribute::SecureConnectionStart => self.secure_connection_start = now,
            ResourceAttribute::ResponseEnd => self.response_end = now,
            ResourceAttribute::StartTime(val) => match val {
                ResourceTimeValue::RedirectStart
                    if self.redirect_start.is_none() || !self.timing_check_passed => {},
                _ => self.start_time = self.get_time_value(val),
            },
        }
    }

    fn get_time_value(&self, time: ResourceTimeValue) -> Option<CrossProcessInstant> {
        match time {
            ResourceTimeValue::Zero => None,
            ResourceTimeValue::Now => Some(CrossProcessInstant::now()),
            ResourceTimeValue::FetchStart => self.fetch_start,
            ResourceTimeValue::RedirectStart => self.redirect_start,
        }
    }

    pub fn mark_timing_check_failed(&mut self) {
        self.timing_check_passed = false;
        self.domain_lookup_start = None;
        self.redirect_count = 0;
        self.request_start = None;
        self.response_start = None;
        self.redirect_start = None;
        self.connect_start = None;
        self.connect_end = None;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, MallocSizeOf)]
/// A simple container of [`ResourceFetchTiming`] to allow easy sharing between threads and allow to set multiple attributes in sequence.
pub struct ResourceFetchTimingContainer(
    #[conditional_malloc_size_of] Arc<Mutex<ResourceFetchTiming>>,
);

impl ResourceFetchTimingContainer {
    pub fn set_attribute(&self, attribute: ResourceAttribute) {
        self.0.lock().set_attribute(attribute);
    }

    /// Set multiple attributes in sequence.
    pub fn set_attributes(&self, attributes: &[ResourceAttribute]) {
        let mut inner = self.0.lock();
        for attribute in attributes {
            inner.set_attribute(attribute.clone());
        }
    }

    pub fn inner(&self) -> MutexGuard<'_, ResourceFetchTiming> {
        self.0.lock()
    }
}

impl From<ResourceFetchTiming> for ResourceFetchTimingContainer {
    fn from(value: ResourceFetchTiming) -> Self {
        ResourceFetchTimingContainer(Arc::new(Mutex::new(value)))
    }
}
