/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS Easing Functions.
//! https://drafts.csswg.org/css-easing/#timing-functions

use crate::parser::ParserContext;
use crate::values::CSSFloat;

/// A generic easing function.
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[value_info(ty = "TIMING_FUNCTION")]
#[repr(u8, C)]
pub enum TimingFunction<Integer, Number> {
    /// `linear | ease | ease-in | ease-out | ease-in-out`
    Keyword(TimingKeyword),
    /// `cubic-bezier(<number>, <number>, <number>, <number>)`
    #[allow(missing_docs)]
    #[css(comma, function)]
    CubicBezier {
        x1: Number,
        y1: Number,
        x2: Number,
        y2: Number,
    },
    /// `step-start | step-end | steps(<integer>, [ <step-position> ]?)`
    /// `<step-position> = jump-start | jump-end | jump-none | jump-both | start | end`
    #[css(comma, function)]
    #[value_info(other_values = "step-start,step-end")]
    Steps(Integer, #[css(skip_if = "is_end")] StepPosition),
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum TimingKeyword {
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[cfg(feature = "gecko")]
fn step_position_jump_enabled(_context: &ParserContext) -> bool {
    use crate::gecko_bindings::structs;
    unsafe { structs::StaticPrefs_sVarCache_layout_css_step_position_jump_enabled }
}

#[cfg(feature = "servo")]
fn step_position_jump_enabled(_context: &ParserContext) -> bool {
    false
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum StepPosition {
    #[parse(condition = "step_position_jump_enabled")]
    JumpStart,
    #[parse(condition = "step_position_jump_enabled")]
    JumpEnd,
    #[parse(condition = "step_position_jump_enabled")]
    JumpNone,
    #[parse(condition = "step_position_jump_enabled")]
    JumpBoth,
    Start,
    End,
}

#[inline]
fn is_end(position: &StepPosition) -> bool {
    *position == StepPosition::JumpEnd || *position == StepPosition::End
}

impl<Integer, Number> TimingFunction<Integer, Number> {
    /// `ease`
    #[inline]
    pub fn ease() -> Self {
        TimingFunction::Keyword(TimingKeyword::Ease)
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
