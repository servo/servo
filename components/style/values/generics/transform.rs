/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use std::fmt;
use style_traits::{HasViewportPercentage, ToCss};
use values::CSSFloat;

/// A generic 2D transformation matrix.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
#[css(comma, function)]
pub struct Matrix<T, U = T> {
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
    pub e: U,
    pub f: U,
}

/// A generic transform origin.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub struct TransformOrigin<H, V, Depth> {
    /// The horizontal origin.
    pub horizontal: H,
    /// The vertical origin.
    pub vertical: V,
    /// The depth.
    pub depth: Depth,
}

/// A generic timing function.
///
/// https://drafts.csswg.org/css-timing-1/#single-timing-function-production
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimingFunction<Integer, Number> {
    /// `linear | ease | ease-in | ease-out | ease-in-out`
    Keyword(TimingKeyword),
    /// `cubic-bezier(<number>, <number>, <number>, <number>)`
    #[allow(missing_docs)]
    CubicBezier { x1: Number, y1: Number, x2: Number, y2: Number },
    /// `step-start | step-end | steps(<integer>, [ start | end ]?)`
    Steps(Integer, StepPosition),
    /// `frames(<integer>)`
    Frames(Integer),
}

define_css_keyword_enum! { TimingKeyword:
    "linear" => Linear,
    "ease" => Ease,
    "ease-in" => EaseIn,
    "ease-out" => EaseOut,
    "ease-in-out" => EaseInOut,
}
add_impls_for_keyword_enum!(TimingKeyword);

define_css_keyword_enum! { StepPosition:
    "start" => Start,
    "end" => End,
}
add_impls_for_keyword_enum!(StepPosition);

impl<H, V, D> TransformOrigin<H, V, D> {
    /// Returns a new transform origin.
    pub fn new(horizontal: H, vertical: V, depth: D) -> Self {
        Self {
            horizontal: horizontal,
            vertical: vertical,
            depth: depth,
        }
    }
}

impl<I, N> HasViewportPercentage for TimingFunction<I, N> {
    fn has_viewport_percentage(&self) -> bool { false }
}

impl<Integer, Number> TimingFunction<Integer, Number> {
    /// `ease`
    #[inline]
    pub fn ease() -> Self {
        TimingFunction::Keyword(TimingKeyword::Ease)
    }
}

impl<Integer, Number> ToCss for TimingFunction<Integer, Number>
where
    Integer: ToCss,
    Number: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            TimingFunction::Keyword(keyword) => keyword.to_css(dest),
            TimingFunction::CubicBezier { ref x1, ref y1, ref x2, ref y2 } => {
                dest.write_str("cubic-bezier(")?;
                x1.to_css(dest)?;
                dest.write_str(", ")?;
                y1.to_css(dest)?;
                dest.write_str(", ")?;
                x2.to_css(dest)?;
                dest.write_str(", ")?;
                y2.to_css(dest)?;
                dest.write_str(")")
            },
            TimingFunction::Steps(ref intervals, position) => {
                dest.write_str("steps(")?;
                intervals.to_css(dest)?;
                if position != StepPosition::End {
                    dest.write_str(", ")?;
                    position.to_css(dest)?;
                }
                dest.write_str(")")
            },
            TimingFunction::Frames(ref frames) => {
                dest.write_str("frames(")?;
                frames.to_css(dest)?;
                dest.write_str(")")
            },
        }
    }
}

impl TimingKeyword {
    /// Returns the keyword as a quadruplet of Bezier point coordinates
    /// `(x1, y1, x2, y2)`.
    #[inline]
    pub fn to_bezier(self) -> (CSSFloat, CSSFloat, CSSFloat, CSSFloat) {
        match self {
            TimingKeyword::Linear => (0., 0., 1., 1.),
            TimingKeyword::Ease => (0.25, 0.1, 0.25, 1.),
            TimingKeyword::EaseIn => (0.42, 0., 1., 1.),
            TimingKeyword::EaseOut => (0., 0., 0.58, 1.),
            TimingKeyword::EaseInOut => (0.42, 0., 0.58, 1.),
        }
    }
}
