/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use style::Atom;
use stylo_atoms::atom;
use taffy::MaxTrackSizingFunction;
use taffy::style_helpers::*;

use super::{as_clamped_i16, stylo};

#[inline]
pub fn length_percentage(val: &stylo::LengthPercentage) -> taffy::LengthPercentage {
    match val.unpack() {
        stylo::UnpackedLengthPercentage::Length(len) => length(len.px()),
        stylo::UnpackedLengthPercentage::Percentage(percentage) => percent(percentage.0),
        stylo::UnpackedLengthPercentage::Calc(calc_ref) => {
            let calc_ptr = calc_ref as *const stylo::CalcLengthPercentage as *const ();
            taffy::LengthPercentage::calc(calc_ptr)
        },
    }
}

#[inline]
fn fit_content_function(limit: &stylo::LengthPercentage) -> taffy::Dimension {
    match limit.unpack() {
        stylo::UnpackedLengthPercentage::Length(length) => {
            taffy::Dimension::fit_content_px(length.px())
        },
        stylo::UnpackedLengthPercentage::Percentage(percentage) => {
            taffy::Dimension::fit_content_percent(percentage.0)
        },
        stylo::UnpackedLengthPercentage::Calc(calc_ref) => {
            let calc_ptr = calc_ref as *const stylo::CalcLengthPercentage as *const ();
            taffy::Dimension::fit_content_calc(calc_ptr)
        },
    }
}

#[inline]
pub fn dimension(value: &stylo::Size) -> taffy::Dimension {
    match value {
        stylo::Size::LengthPercentage(value) => length_percentage(&value.0).into(),
        stylo::Size::Auto => taffy::Dimension::AUTO,
        stylo::Size::MaxContent => taffy::Dimension::max_content(),
        stylo::Size::MinContent => taffy::Dimension::min_content(),
        stylo::Size::FitContent => taffy::Dimension::fit_content(),
        stylo::Size::FitContentFunction(limit) => fit_content_function(&limit.0),
        stylo::Size::Stretch | stylo::Size::WebkitFillAvailable => taffy::Dimension::stretch(),

        // Anchor positioning will be flagged off for time being
        stylo::Size::AnchorSizeFunction(_) => unreachable!(),
        stylo::Size::AnchorContainingCalcFunction(_) => unreachable!(),
    }
}

#[inline]
pub fn min_size(value: &stylo::Size) -> taffy::LengthPercentageAuto {
    match value {
        stylo::Size::LengthPercentage(value) => length_percentage(&value.0).into(),
        stylo::Size::Auto => taffy::LengthPercentageAuto::AUTO,

        // TODO: implement other values in Taffy
        stylo::Size::MaxContent => taffy::LengthPercentageAuto::AUTO,
        stylo::Size::MinContent => taffy::LengthPercentageAuto::AUTO,
        stylo::Size::FitContent => taffy::LengthPercentageAuto::AUTO,
        stylo::Size::FitContentFunction(_) => taffy::LengthPercentageAuto::AUTO,
        stylo::Size::Stretch | stylo::Size::WebkitFillAvailable => {
            taffy::LengthPercentageAuto::AUTO
        },

        // Anchor positioning will be flagged off for time being
        stylo::Size::AnchorSizeFunction(_) | stylo::Size::AnchorContainingCalcFunction(_) => {
            unreachable!("Anchor positioning is disabled in stylo")
        },
    }
}

#[inline]
pub fn max_size(value: &stylo::MaxSize) -> taffy::LengthPercentageAuto {
    match value {
        stylo::MaxSize::LengthPercentage(value) => length_percentage(&value.0).into(),
        stylo::MaxSize::None => taffy::LengthPercentageAuto::AUTO,

        // TODO: implement other values in Taffy
        stylo::MaxSize::MaxContent => taffy::LengthPercentageAuto::AUTO,
        stylo::MaxSize::MinContent => taffy::LengthPercentageAuto::AUTO,
        stylo::MaxSize::FitContent => taffy::LengthPercentageAuto::AUTO,
        stylo::MaxSize::FitContentFunction(_) => taffy::LengthPercentageAuto::AUTO,
        stylo::MaxSize::Stretch | stylo::MaxSize::WebkitFillAvailable => {
            taffy::LengthPercentageAuto::AUTO
        },

        // Anchor positioning will be flagged off for time being
        stylo::MaxSize::AnchorSizeFunction(_) | stylo::MaxSize::AnchorContainingCalcFunction(_) => {
            unreachable!("Anchor positioning is disabled in stylo")
        },
    }
}

#[inline]
pub fn margin(val: &stylo::MarginVal) -> taffy::LengthPercentageAuto {
    match val {
        stylo::MarginVal::Auto => taffy::LengthPercentageAuto::AUTO,
        stylo::MarginVal::LengthPercentage(val) => length_percentage(val).into(),

        // Anchor positioning will be flagged off for time being
        stylo::MarginVal::AnchorSizeFunction(_) |
        stylo::MarginVal::AnchorContainingCalcFunction(_) => {
            unreachable!("Anchor positioning is disabled in stylo")
        },
    }
}

#[inline]
pub fn inset(val: &stylo::InsetVal) -> taffy::LengthPercentageAuto {
    match val {
        stylo::InsetVal::Auto => taffy::LengthPercentageAuto::AUTO,
        stylo::InsetVal::LengthPercentage(val) => length_percentage(val).into(),

        // Anchor positioning will be flagged off for time being
        stylo::InsetVal::AnchorSizeFunction(_) |
        stylo::InsetVal::AnchorFunction(_) |
        stylo::InsetVal::AnchorContainingCalcFunction(_) => {
            unreachable!("Anchor positioning is disabled in stylo")
        },
    }
}

#[inline]
pub fn is_block(input: stylo::Display) -> bool {
    matches!(input.outside(), stylo::DisplayOutside::Block) &&
        matches!(
            input.inside(),
            stylo::DisplayInside::Flow | stylo::DisplayInside::FlowRoot
        )
}

#[inline]
pub fn box_generation_mode(input: stylo::Display) -> taffy::BoxGenerationMode {
    match input.inside() {
        stylo::DisplayInside::None => taffy::BoxGenerationMode::None,
        _ => taffy::BoxGenerationMode::Normal,
    }
}

#[inline]
pub fn box_sizing(input: stylo::BoxSizing) -> taffy::BoxSizing {
    match input {
        stylo::BoxSizing::BorderBox => taffy::BoxSizing::BorderBox,
        stylo::BoxSizing::ContentBox => taffy::BoxSizing::ContentBox,
    }
}

#[inline]
pub fn direction(input: stylo::Direction) -> taffy::Direction {
    match input {
        stylo::Direction::Ltr => taffy::Direction::Ltr,
        stylo::Direction::Rtl => taffy::Direction::Rtl,
    }
}

#[inline]
pub fn position(input: stylo::Position) -> taffy::Position {
    match input {
        stylo::Position::Static => taffy::Position::Static,
        stylo::Position::Relative => taffy::Position::Relative,
        stylo::Position::Absolute => taffy::Position::Absolute,
        stylo::Position::Fixed => taffy::Position::Fixed,
        stylo::Position::Sticky => taffy::Position::Sticky,
    }
}

#[inline]
pub fn overflow(input: stylo::Overflow) -> taffy::Overflow {
    // TODO: Enable Overflow::Clip in servo configuration of stylo
    match input {
        stylo::Overflow::Visible => taffy::Overflow::Visible,
        stylo::Overflow::Hidden => taffy::Overflow::Hidden,
        stylo::Overflow::Scroll => taffy::Overflow::Scroll,
        stylo::Overflow::Clip => taffy::Overflow::Clip,
        // TODO: Support Overflow::Auto in Taffy
        stylo::Overflow::Auto => taffy::Overflow::Scroll,
    }
}

#[inline]
pub fn aspect_ratio(input: stylo::AspectRatio) -> Option<f32> {
    match input.ratio {
        stylo::PreferredRatio::None => None,
        stylo::PreferredRatio::Ratio(val) => Some(val.0.0 / val.1.0),
    }
}

#[inline]
fn alignment_safety(flags: stylo::AlignFlags) -> taffy::AlignmentSafety {
    match flags {
        stylo::AlignFlags::SAFE => taffy::AlignmentSafety::Safe,
        stylo::AlignFlags::UNSAFE => taffy::AlignmentSafety::Unsafe,
        _ => taffy::AlignmentSafety::Default,
    }
}

#[inline]
pub fn content_alignment(input: stylo::ContentDistribution) -> taffy::AlignContent {
    let keyword = match input.primary().value() {
        stylo::AlignFlags::NORMAL => taffy::AlignContentKeyword::Normal,
        stylo::AlignFlags::AUTO => taffy::AlignContentKeyword::Normal,
        stylo::AlignFlags::START => taffy::AlignContentKeyword::Start,
        stylo::AlignFlags::END => taffy::AlignContentKeyword::End,
        stylo::AlignFlags::LEFT => taffy::AlignContentKeyword::Start,
        stylo::AlignFlags::RIGHT => taffy::AlignContentKeyword::End,
        stylo::AlignFlags::FLEX_START => taffy::AlignContentKeyword::FlexStart,
        stylo::AlignFlags::STRETCH => taffy::AlignContentKeyword::Stretch,
        stylo::AlignFlags::FLEX_END => taffy::AlignContentKeyword::FlexEnd,
        stylo::AlignFlags::CENTER => taffy::AlignContentKeyword::Center,
        stylo::AlignFlags::SPACE_BETWEEN => taffy::AlignContentKeyword::SpaceBetween,
        stylo::AlignFlags::SPACE_AROUND => taffy::AlignContentKeyword::SpaceAround,
        stylo::AlignFlags::SPACE_EVENLY => taffy::AlignContentKeyword::SpaceEvenly,
        // Should never be hit. But no real reason to panic here.
        _ => taffy::AlignContentKeyword::Normal,
    };

    taffy::AlignContent {
        keyword,
        safety: alignment_safety(input.primary().flags()),
    }
}

#[inline]
pub fn items_alignment(input: stylo::AlignFlags) -> taffy::AlignItems {
    self_alignment(input).unwrap_or(taffy::AlignItems::NORMAL)
}

#[inline]
pub fn self_alignment(input: stylo::AlignFlags) -> Option<taffy::AlignSelf> {
    let keyword = match input.value() {
        stylo::AlignFlags::AUTO => return None,
        stylo::AlignFlags::NORMAL => taffy::AlignItemsKeyword::Normal,
        stylo::AlignFlags::STRETCH => taffy::AlignItemsKeyword::Stretch,
        stylo::AlignFlags::FLEX_START => taffy::AlignItemsKeyword::FlexStart,
        stylo::AlignFlags::FLEX_END => taffy::AlignItemsKeyword::FlexEnd,
        stylo::AlignFlags::SELF_START => taffy::AlignItemsKeyword::Start,
        stylo::AlignFlags::SELF_END => taffy::AlignItemsKeyword::End,
        stylo::AlignFlags::START => taffy::AlignItemsKeyword::Start,
        stylo::AlignFlags::END => taffy::AlignItemsKeyword::End,
        stylo::AlignFlags::LEFT => taffy::AlignItemsKeyword::Start,
        stylo::AlignFlags::RIGHT => taffy::AlignItemsKeyword::End,
        stylo::AlignFlags::CENTER => taffy::AlignItemsKeyword::Center,
        stylo::AlignFlags::BASELINE => taffy::AlignItemsKeyword::Baseline,
        // Should never be hit. But no real reason to panic here.
        _ => return None,
    };

    Some(taffy::AlignItems {
        keyword,
        safety: alignment_safety(input.flags()),
    })
}

#[inline]
pub fn gap(input: &stylo::Gap) -> taffy::LengthPercentage {
    match input {
        // For Flexbox and CSS Grid the "normal" value is 0px. This may need to be updated
        // if we ever implement multi-column layout.
        stylo::Gap::Normal => taffy::LengthPercentage::ZERO,
        stylo::Gap::LengthPercentage(val) => length_percentage(&val.0),
    }
}

// CSS Grid styles
// ===============

#[inline]
pub fn grid_auto_flow(input: stylo::GridAutoFlow) -> taffy::GridAutoFlow {
    let is_row = input.contains(stylo::GridAutoFlow::ROW);
    let is_dense = input.contains(stylo::GridAutoFlow::DENSE);

    match (is_row, is_dense) {
        (true, false) => taffy::GridAutoFlow::Row,
        (true, true) => taffy::GridAutoFlow::RowDense,
        (false, false) => taffy::GridAutoFlow::Column,
        (false, true) => taffy::GridAutoFlow::ColumnDense,
    }
}

#[inline]
pub fn grid_line(input: &stylo::GridLine) -> taffy::GridPlacement<Atom> {
    if input.is_auto() {
        taffy::GridPlacement::Auto
    } else if input.is_span {
        if input.ident.0 != atom!("") {
            taffy::GridPlacement::NamedSpan(
                input.ident.0.clone(),
                as_clamped_i16(input.line_num) as u16,
            )
        } else {
            taffy::GridPlacement::Span(input.line_num as u16)
        }
    } else if input.ident.0 != atom!("") {
        taffy::GridPlacement::NamedLine(input.ident.0.clone(), as_clamped_i16(input.line_num))
    } else if input.line_num != 0 {
        taffy::style_helpers::line(as_clamped_i16(input.line_num))
    } else {
        taffy::GridPlacement::Auto
    }
}

#[inline]
pub fn track_repeat(input: stylo::RepeatCount<i32>) -> taffy::RepetitionCount {
    match input {
        stylo::RepeatCount::Number(count) => {
            taffy::RepetitionCount::Count(as_clamped_i16(count) as u16)
        },
        stylo::RepeatCount::AutoFill => taffy::RepetitionCount::AutoFill,
        stylo::RepeatCount::AutoFit => taffy::RepetitionCount::AutoFit,
    }
}

#[inline]
pub fn track_size(input: &stylo::TrackSize<stylo::LengthPercentage>) -> taffy::TrackSizingFunction {
    match input {
        stylo::TrackSize::Breadth(breadth) => taffy::MinMax {
            min: min_track(breadth),
            max: max_track(breadth),
        },
        stylo::TrackSize::Minmax(min, max) => taffy::MinMax {
            min: min_track(min),
            max: max_track(max),
        },
        stylo::TrackSize::FitContent(limit) => taffy::MinMax {
            min: taffy::MinTrackSizingFunction::AUTO,
            max: match limit {
                stylo::TrackBreadth::Breadth(lp) => {
                    MaxTrackSizingFunction::fit_content(length_percentage(lp))
                },

                // Are these valid? Taffy doesn't support this in any case
                stylo::TrackBreadth::Flex(_) => unreachable!(),
                stylo::TrackBreadth::Auto => unreachable!(),
                stylo::TrackBreadth::MinContent => unreachable!(),
                stylo::TrackBreadth::MaxContent => unreachable!(),
            },
        },
    }
}

#[inline]
pub fn min_track(
    input: &stylo::TrackBreadth<stylo::LengthPercentage>,
) -> taffy::MinTrackSizingFunction {
    match input {
        stylo::TrackBreadth::Breadth(lp) => length_percentage(lp).into(),
        stylo::TrackBreadth::Flex(_) => taffy::MinTrackSizingFunction::AUTO,
        stylo::TrackBreadth::Auto => taffy::MinTrackSizingFunction::AUTO,
        stylo::TrackBreadth::MinContent => taffy::MinTrackSizingFunction::MIN_CONTENT,
        stylo::TrackBreadth::MaxContent => taffy::MinTrackSizingFunction::MAX_CONTENT,
    }
}

#[inline]
pub fn max_track(
    input: &stylo::TrackBreadth<stylo::LengthPercentage>,
) -> taffy::MaxTrackSizingFunction {
    match input {
        stylo::TrackBreadth::Breadth(lp) => length_percentage(lp).into(),
        stylo::TrackBreadth::Flex(val) => fr(val.0),
        stylo::TrackBreadth::Auto => taffy::MaxTrackSizingFunction::AUTO,
        stylo::TrackBreadth::MinContent => taffy::MaxTrackSizingFunction::MIN_CONTENT,
        stylo::TrackBreadth::MaxContent => taffy::MaxTrackSizingFunction::MAX_CONTENT,
    }
}
