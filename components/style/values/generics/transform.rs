/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values that are related to transformations.

use euclid::Point2D;
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
    CubicBezier(Point2D<Number>, Point2D<Number>),
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
            TimingFunction::CubicBezier(ref p1, ref p2) => {
                dest.write_str("cubic-bezier(")?;
                p1.x.to_css(dest)?;
                dest.write_str(", ")?;
                p1.y.to_css(dest)?;
                dest.write_str(", ")?;
                p2.x.to_css(dest)?;
                dest.write_str(", ")?;
                p2.y.to_css(dest)?;
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
    /// Returns this timing keyword as a pair of `cubic-bezier()` points.
    #[inline]
    pub fn to_bezier_points(self) -> (Point2D<CSSFloat>, Point2D<CSSFloat>) {
        match self {
            TimingKeyword::Linear => (Point2D::new(0., 0.), Point2D::new(1., 1.)),
            TimingKeyword::Ease => (Point2D::new(0.25, 0.1), Point2D::new(0.25, 1.)),
            TimingKeyword::EaseIn => (Point2D::new(0.42, 0.), Point2D::new(1., 1.)),
            TimingKeyword::EaseOut => (Point2D::new(0., 0.), Point2D::new(0.58, 1.)),
            TimingKeyword::EaseInOut => (Point2D::new(0.42, 0.), Point2D::new(0.58, 1.)),
        }
    }
}
