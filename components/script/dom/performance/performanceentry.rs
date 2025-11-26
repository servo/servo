/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use strum::VariantArray;
use time::Duration;

use super::performance::ToDOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector};
use crate::dom::bindings::str::DOMString;

/// All supported entry types, in alphabetical order.
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq, VariantArray)]
pub(crate) enum EntryType {
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

#[dom_struct]
pub(crate) struct PerformanceEntry {
    reflector_: Reflector,
    name: DOMString,
    entry_type: EntryType,
    #[no_trace]
    start_time: Option<CrossProcessInstant>,
    /// The duration of this [`PerformanceEntry`]. This is a [`time::Duration`],
    /// because it can be negative and `std::time::Duration` cannot be.
    #[no_trace]
    #[ignore_malloc_size_of = "No MallocSizeOf support for `time` crate"]
    duration: Duration,
}

impl PerformanceEntry {
    pub(crate) fn new_inherited(
        name: DOMString,
        entry_type: EntryType,
        start_time: Option<CrossProcessInstant>,
        duration: Duration,
    ) -> PerformanceEntry {
        PerformanceEntry {
            reflector_: Reflector::new(),
            name,
            entry_type,
            start_time,
            duration,
        }
    }

    pub(crate) fn entry_type(&self) -> EntryType {
        self.entry_type
    }

    pub(crate) fn name(&self) -> &DOMString {
        &self.name
    }

    pub(crate) fn start_time(&self) -> Option<CrossProcessInstant> {
        self.start_time
    }

    pub(crate) fn duration(&self) -> Duration {
        self.duration
    }
}

impl PerformanceEntryMethods<crate::DomTypeHolder> for PerformanceEntry {
    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-entrytype>
    fn EntryType(&self) -> DOMString {
        DOMString::from(self.entry_type.as_str())
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-starttime>
    fn StartTime(&self) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .maybe_to_dom_high_res_time_stamp(self.start_time)
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceentry-duration>
    fn Duration(&self) -> DOMHighResTimeStamp {
        self.duration.to_dom_high_res_time_stamp()
    }
}
