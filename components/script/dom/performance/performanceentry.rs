/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::rust::Trace;
use script_bindings::domstring::TracedDOMString;
use script_bindings::inheritance::Castable;
use script_bindings::num::Finite;
use script_bindings::reflector::Reflector;
use servo_base::cross_process_instant::CrossProcessInstant;
use servo_url::ServoUrl;
use strum::VariantArray;
use time::Duration;

use super::performance::ToDOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceResourceTimingBinding::PerformanceResourceTimingMethods;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::DOMString;
use crate::dom::performance::performanceresourcetiming::PerformanceResourceTiming;

/// All supported entry types, in alphabetical order.
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq, VariantArray)]
pub(crate) enum EntryType {
    ContainerTiming,
    LargestContentfulPaint,
    Mark,
    Measure,
    Navigation,
    Paint,
    Resource,
    VisibilityState,
}

impl EntryType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            EntryType::ContainerTiming => "container",
            EntryType::Measure => "measure",
            EntryType::Mark => "mark",
            EntryType::LargestContentfulPaint => "largest-contentful-paint",
            EntryType::Paint => "paint",
            EntryType::Navigation => "navigation",
            EntryType::Resource => "resource",
            EntryType::VisibilityState => "visibility-state",
        }
    }
}

impl<'a> TryFrom<&'a str> for EntryType {
    type Error = ();

    fn try_from(value: &'a str) -> Result<EntryType, ()> {
        Ok(match value {
            "container" => EntryType::ContainerTiming,
            "measure" => EntryType::Measure,
            "mark" => EntryType::Mark,
            "largest-contentful-paint" => EntryType::LargestContentfulPaint,
            "paint" => EntryType::Paint,
            "navigation" => EntryType::Navigation,
            "resource" => EntryType::Resource,
            "visibility-state" => EntryType::VisibilityState,
            _ => return Err(()),
        })
    }
}

#[derive(MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum NameOrUrl {
    Url(ServoUrl),
    Name(TracedDOMString),
}

#[expect(unsafe_code)]
unsafe impl Trace for NameOrUrl {
    unsafe fn trace(&self, trc: *mut js::jsapi::JSTracer) {
        match self {
            NameOrUrl::Url(_) => {},
            NameOrUrl::Name(traced_domstring) => unsafe { traced_domstring.trace(trc) },
        }
    }
}

impl From<ServoUrl> for NameOrUrl {
    fn from(value: ServoUrl) -> Self {
        NameOrUrl::Url(value)
    }
}

impl From<DOMString> for NameOrUrl {
    fn from(value: DOMString) -> Self {
        NameOrUrl::Name(value.traced())
    }
}

/// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry>
#[dom_struct]
pub(crate) struct PerformanceEntry {
    reflector_: Reflector,

    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-name>
    /// The name of this entry or the URL. This conversion is done lazily in order
    /// to optimize the size of the [`PerformanceEntry`].
    name: RefCell<NameOrUrl>,

    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-entrytype>
    entry_type: EntryType,

    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-starttime>
    #[no_trace]
    start_time: Option<CrossProcessInstant>,

    /// The duration of this [`PerformanceEntry`]. This is a [`time::Duration`],
    /// because it can be negative and `std::time::Duration` cannot be.
    ///
    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-duration>
    #[no_trace]
    #[ignore_malloc_size_of = "No MallocSizeOf support for `time` crate"]
    duration: Duration,
}

impl PerformanceEntry {
    #[cfg_attr(
        crown,
        expect(
            crown::unrooted_must_root,
            reason = "Not rooting this value is fine, we do not have a `&mut JSContext`, hence, no GC can happen"
        )
    )]
    pub(crate) fn new_inherited(
        name_or_url: NameOrUrl,
        entry_type: EntryType,
        start_time: Option<CrossProcessInstant>,
        duration: Duration,
    ) -> PerformanceEntry {
        PerformanceEntry {
            reflector_: Reflector::new(),
            name: name_or_url.into(),
            entry_type,
            start_time,
            duration,
        }
    }

    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-name>
    pub(crate) fn entry_type(&self) -> EntryType {
        self.entry_type
    }

    /// <https://www.w3.org/TR/performance-timeline/#dom-performanceentry-starttime>
    pub(crate) fn start_time(&self) -> Option<CrossProcessInstant> {
        self.start_time
    }
}

impl PerformanceEntryMethods<crate::DomTypeHolder> for PerformanceEntry {
    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-name>
    fn Name(&self) -> DOMString {
        {
            if let NameOrUrl::Name(name) = &*self.name.borrow() {
                return name.clone().root();
            }
            let mut name = self.name.borrow_mut();

            if let NameOrUrl::Url(url) = &*name {
                *name = NameOrUrl::Name(url.to_string().into());
            }
        }
        self.Name()
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-entrytype>
    fn EntryType(&self) -> DOMString {
        DOMString::from(self.entry_type.as_str())
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-starttime>
    fn StartTime(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        self.global()
            .performance(cx)
            .maybe_to_dom_high_res_time_stamp(self.start_time)
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-duration>
    fn Duration(&self, cx: &mut JSContext) -> DOMHighResTimeStamp {
        if let Some(resource_timing) = self.downcast::<PerformanceResourceTiming>() {
            Finite::new(*resource_timing.ResponseEnd(cx) - *self.StartTime(cx))
                .unwrap_or(Finite::default())
        } else {
            self.duration.to_dom_high_res_time_stamp()
        }
    }
}
