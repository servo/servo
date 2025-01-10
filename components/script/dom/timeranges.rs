/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::TimeRangesBinding::TimeRangesMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[derive(Clone, JSTraceable, MallocSizeOf)]
struct TimeRange {
    start: f64,
    end: f64,
}

impl TimeRange {
    pub(crate) fn union(&mut self, other: &TimeRange) {
        self.start = f64::min(self.start, other.start);
        self.end = f64::max(self.end, other.end);
    }

    fn contains(&self, time: f64) -> bool {
        self.start <= time && time < self.end
    }

    fn is_overlapping(&self, other: &TimeRange) -> bool {
        // This also covers the case where `self` is entirely contained within `other`,
        // for example: `self` = [2,3) and `other` = [1,4).
        self.contains(other.start) || self.contains(other.end) || other.contains(self.start)
    }

    fn is_contiguous(&self, other: &TimeRange) -> bool {
        other.start == self.end || other.end == self.start
    }

    pub(crate) fn is_before(&self, other: &TimeRange) -> bool {
        other.start >= self.end
    }
}

impl fmt::Debug for TimeRange {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "[{},{})", self.start, self.end)
    }
}

#[derive(Debug)]
pub enum TimeRangesError {
    EndOlderThanStart,
    OutOfRange,
}

#[derive(Clone, Debug, Default, JSTraceable, MallocSizeOf)]
pub struct TimeRangesContainer {
    ranges: Vec<TimeRange>,
}

impl TimeRangesContainer {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.ranges.len() as u32
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    pub fn start(&self, index: u32) -> Result<f64, TimeRangesError> {
        self.ranges
            .get(index as usize)
            .map(|r| r.start)
            .ok_or(TimeRangesError::OutOfRange)
    }

    pub fn end(&self, index: u32) -> Result<f64, TimeRangesError> {
        self.ranges
            .get(index as usize)
            .map(|r| r.end)
            .ok_or(TimeRangesError::OutOfRange)
    }

    pub fn add(&mut self, start: f64, end: f64) -> Result<(), TimeRangesError> {
        if start > end {
            return Err(TimeRangesError::EndOlderThanStart);
        }

        let mut new_range = TimeRange { start, end };

        // For each present range check if we need to:
        // - merge with the added range, in case we are overlapping or contiguous,
        // - insert in place, we are completely, not overlapping and not contiguous
        //   in between two ranges.
        let mut idx = 0;
        while idx < self.ranges.len() {
            if new_range.is_overlapping(&self.ranges[idx]) ||
                new_range.is_contiguous(&self.ranges[idx])
            {
                // The ranges are either overlapping or contiguous,
                // we need to merge the new range with the existing one.
                new_range.union(&self.ranges[idx]);
                self.ranges.remove(idx);
            } else if new_range.is_before(&self.ranges[idx]) &&
                (idx == 0 || self.ranges[idx - 1].is_before(&new_range))
            {
                // We are exactly after the current previous range and before the current
                // range, while not overlapping with none of them.
                // Or we are simply at the beginning.
                self.ranges.insert(idx, new_range);
                return Ok(());
            } else {
                idx += 1;
            }
        }

        // Insert at the end.
        self.ranges.insert(idx, new_range);

        Ok(())
    }
}

#[dom_struct]
pub(crate) struct TimeRanges {
    reflector_: Reflector,
    ranges: TimeRangesContainer,
}

impl TimeRanges {
    fn new_inherited(ranges: TimeRangesContainer) -> TimeRanges {
        Self {
            reflector_: Reflector::new(),
            ranges,
        }
    }

    pub(crate) fn new(window: &Window, ranges: TimeRangesContainer) -> DomRoot<TimeRanges> {
        reflect_dom_object(
            Box::new(TimeRanges::new_inherited(ranges)),
            window,
            CanGc::note(),
        )
    }
}

impl TimeRangesMethods<crate::DomTypeHolder> for TimeRanges {
    // https://html.spec.whatwg.org/multipage/#dom-timeranges-length
    fn Length(&self) -> u32 {
        self.ranges.len()
    }

    // https://html.spec.whatwg.org/multipage/#dom-timeranges-start
    fn Start(&self, index: u32) -> Fallible<Finite<f64>> {
        self.ranges
            .start(index)
            .map(Finite::wrap)
            .map_err(|_| Error::IndexSize)
    }

    // https://html.spec.whatwg.org/multipage/#dom-timeranges-end
    fn End(&self, index: u32) -> Fallible<Finite<f64>> {
        self.ranges
            .end(index)
            .map(Finite::wrap)
            .map_err(|_| Error::IndexSize)
    }
}
