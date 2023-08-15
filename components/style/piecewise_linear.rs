/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A piecewise linear function, following CSS linear easing
/// draft as in https://github.com/w3c/csswg-drafts/pull/6533.
use euclid::approxeq::ApproxEq;
use itertools::Itertools;

use crate::values::CSSFloat;

type ValueType = CSSFloat;
/// a single entry in a piecewise linear function.
#[derive(Clone, Copy)]
#[repr(C)]
struct Entry {
    x: ValueType,
    y: ValueType,
}

/// Representation of a piecewise linear function, a series of linear functions.
#[derive(Default)]
#[repr(C)]
pub struct PiecewiseLinearFunction {
    entries: crate::OwnedSlice<Entry>,
}

impl PiecewiseLinearFunction {
    /// Interpolate y value given x and two points. The linear function will be rooted at the asymptote.
    fn interpolate(x: ValueType, prev: Entry, next: Entry, asymptote: &Entry) -> ValueType {
        // Line is vertical, or the two points are identical. Avoid infinite slope by pretending
        // the line is flat.
        if prev.x.approx_eq(&next.x) {
            return asymptote.y;
        }
        let slope = (next.y - prev.y) / (next.x - prev.x);
        return slope * (x - asymptote.x) + asymptote.y;
    }

    /// Get the y value of the piecewise linear function given the x value.
    pub fn at(&self, x: ValueType) -> ValueType {
        if !x.is_finite() {
            return if x > 0.0 { 1.0 } else { 0.0 };
        }
        if self.entries.is_empty() {
            // Implied y = x, as per spec.
            return x;
        }
        if self.entries.len() == 1 {
            // Implied y = <constant>, as per spec.
            return self.entries[0].y;
        }
        // Spec dictates the valid input domain is [0, 1]. Outside of this range, the output
        // should be calculated as if the slopes at start and end extend to infinity. However, if the
        // start/end have two points of the same position, the line should extend along the x-axis.
        // The function doesn't have to cover the input domain, in which case the extension logic
        // applies even if the input falls in the input domain.
        // Also, we're guaranteed to have at least two elements at this point.
        if x < self.entries[0].x {
            return Self::interpolate(x, self.entries[0], self.entries[1], &self.entries[0]);
        }
        let mut rev_iter = self.entries.iter().rev();
        let last = rev_iter.next().unwrap();
        if x > last.x {
            let second_last = rev_iter.next().unwrap();
            return Self::interpolate(x, *second_last, *last, last);
        }

        // Now we know the input sits within the domain explicitly defined by our function.
        for (prev, next) in self.entries.iter().tuple_windows() {
            if x > next.x {
                continue;
            }
            // Prefer left hand side value
            if x.approx_eq(&prev.x) {
                return prev.y;
            }
            if x.approx_eq(&next.x) {
                return next.y;
            }
            return Self::interpolate(x, *prev, *next, prev);
        }
        unreachable!("Input is supposed to be within the entries' min & max!");
    }
}

/// Entry of a piecewise linear function while building, where the calculation of x value can be deferred.
#[derive(Clone, Copy)]
struct BuildEntry {
    x: Option<ValueType>,
    y: ValueType,
}

/// Builder object to generate a linear function.
#[derive(Default)]
pub struct PiecewiseLinearFunctionBuilder {
    largest_x: Option<ValueType>,
    smallest_x: Option<ValueType>,
    entries: Vec<BuildEntry>,
}

impl PiecewiseLinearFunctionBuilder {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        PiecewiseLinearFunctionBuilder::default()
    }

    fn create_entry(&mut self, y: ValueType, x: Option<ValueType>) {
        let x = match x {
            Some(x) if x.is_finite() => x,
            _ => {
                self.entries.push(BuildEntry { x: None, y });
                return;
            },
        };
        // Specified x value cannot regress, as per spec.
        let x = match self.largest_x {
            Some(largest_x) => x.max(largest_x),
            None => x,
        };
        self.largest_x = Some(x);
        // Whatever we see the earliest is the smallest value.
        if self.smallest_x.is_none() {
            self.smallest_x = Some(x);
        }
        self.entries.push(BuildEntry { x: Some(x), y });
    }

    /// Add a new entry into the piecewise linear function with specified y value.
    /// If the start x value is given, that is where the x value will be. Otherwise,
    /// the x value is calculated later. If the end x value is specified, a flat segment
    /// is generated. If start x value is not specified but end x is, it is treated as
    /// start x.
    pub fn push(mut self, y: CSSFloat, x_start: Option<CSSFloat>, x_end: Option<CSSFloat>) -> Self {
        self.create_entry(y, x_start);
        if x_end.is_some() {
            self.create_entry(y, x_end.map(|x| x));
        }
        self
    }

    /// Finish building the piecewise linear function by resolving all undefined x values,
    /// then return the result.
    pub fn build(mut self) -> PiecewiseLinearFunction {
        if self.entries.is_empty() {
            return PiecewiseLinearFunction::default();
        }
        if self.entries.len() == 1 {
            // Don't bother resolving anything.
            return PiecewiseLinearFunction {
                entries: crate::OwnedSlice::from_slice(&[Entry {
                    x: 0.,
                    y: self.entries[0].y,
                }]),
            };
        }
        // Guaranteed at least two elements.
        // Start and end elements guaranteed to have defined x value.
        // Note(dshin): Spec asserts that start/end elements are supposed to have 0/1 assigned
        // respectively if their x values are undefined at this time; however, the spec does
        // not disallow negative/100%+ inputs, and inputs like `linear(0, 0.1 -10%, 0.9 110%, 1.0)`
        // would break the assumption that the x values in the list increase monotonically.
        // Otherwise, we still want 0/1 assigned to the start/end values regardless of
        // adjacent x values (i.e. `linear(0, 0.1 10%, 0.9 90%, 1.0)` ==
        // `linear(0 0%, 0.1 10%, 0.9 90%, 1.0)` != `linear(0 10%, 0.1 10%, 0.9 90%, 1.0 90%)`)
        self.entries[0]
            .x
            .get_or_insert(self.smallest_x.filter(|x| x < &0.0).unwrap_or(0.0));
        self.entries
            .last_mut()
            .unwrap()
            .x
            .get_or_insert(self.largest_x.filter(|x| x > &1.0).unwrap_or(1.0));

        let mut result = Vec::with_capacity(self.entries.len());
        result.push(Entry {
            x: self.entries[0].x.unwrap(),
            y: self.entries[0].y,
        });
        for (i, e) in self.entries.iter().enumerate().skip(1) {
            if e.x.is_none() {
                // Need to calculate x values by first finding an entry with the first
                // defined x value (Guaranteed to exist as the list end has it defined).
                continue;
            }
            // x is defined for this element.
            let divisor = i - result.len() + 1;
            // Any element(s) with undefined x to assign?
            if divisor != 1 {
                // Have at least one element in result at all times.
                let start_x = result.last().unwrap().x;
                let increment = (e.x.unwrap() - start_x) / divisor as ValueType;
                // Grab every element with undefined x to this point, which starts at the end of the result
                // array, and ending right before the current index. Then, assigned the evenly divided
                // x values.
                result.extend(
                    self.entries[result.len()..i]
                        .iter()
                        .enumerate()
                        .map(|(j, e)| {
                            debug_assert!(e.x.is_none(), "Expected an entry with x undefined!");
                            Entry {
                                x: increment * (j + 1) as ValueType + start_x,
                                y: e.y,
                            }
                        }),
                );
            }
            result.push(Entry {
                x: e.x.unwrap(),
                y: e.y,
            });
        }
        debug_assert_eq!(
            result.len(),
            self.entries.len(),
            "Should've mapped one-to-one!"
        );
        PiecewiseLinearFunction {
            entries: result.into(),
        }
    }
}
