/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

/// Private module of type aliases so we can refer to stylo types with nicer names
mod stylo {
    pub(crate) use style::properties::generated::longhands::box_sizing::computed_value::T as BoxSizing;
    pub(crate) use style::properties::longhands::aspect_ratio::computed_value::T as AspectRatio;
    pub(crate) use style::properties::longhands::position::computed_value::T as Position;
    pub(crate) use style::values::computed::length_percentage::Unpacked as UnpackedLengthPercentage;
    pub(crate) use style::values::computed::{LengthPercentage, Percentage};
    pub(crate) use style::values::generics::NonNegative;
    pub(crate) use style::values::generics::length::{
        GenericLengthPercentageOrNormal, GenericMargin, GenericMaxSize, GenericSize,
    };
    pub(crate) use style::values::generics::position::{Inset as GenericInset, PreferredRatio};
    pub(crate) use style::values::specified::align::{AlignFlags, ContentDistribution};
    pub(crate) use style::values::specified::box_::{
        Display, DisplayInside, DisplayOutside, Overflow,
    };
    pub(crate) type MarginVal = GenericMargin<LengthPercentage>;
    pub(crate) type InsetVal = GenericInset<Percentage, LengthPercentage>;
    pub(crate) type Size = GenericSize<NonNegative<LengthPercentage>>;
    pub(crate) type MaxSize = GenericMaxSize<NonNegative<LengthPercentage>>;

    pub(crate) type Gap = GenericLengthPercentageOrNormal<NonNegative<LengthPercentage>>;

    pub(crate) use style::computed_values::grid_auto_flow::T as GridAutoFlow;
    pub(crate) use style::values::computed::{GridLine, GridTemplateComponent, ImplicitGridTracks};
    pub(crate) use style::values::generics::grid::{
        RepeatCount, TrackBreadth, TrackListValue, TrackSize,
    };
    pub(crate) use style::values::specified::GenericGridTemplateComponent;
}

use taffy::MaxTrackSizingFunction;
use taffy::style_helpers::*;

#[inline]
pub fn length_percentage(val: &stylo::LengthPercentage) -> taffy::LengthPercentage {
    match val.unpack() {
        stylo::UnpackedLengthPercentage::Length(len) => length(len.px()),
        stylo::UnpackedLengthPercentage::Percentage(percentage) => percent(percentage.0),
        // TODO: Support calc
        stylo::UnpackedLengthPercentage::Calc(_) => percent(0.0),
    }
}

#[inline]
pub fn dimension(val: &stylo::Size) -> taffy::Dimension {
    match val {
        stylo::Size::LengthPercentage(val) => length_percentage(&val.0).into(),
        stylo::Size::Auto => taffy::Dimension::AUTO,

        // TODO: implement other values in Taffy
        stylo::Size::MaxContent => taffy::Dimension::AUTO,
        stylo::Size::MinContent => taffy::Dimension::AUTO,
        stylo::Size::FitContent => taffy::Dimension::AUTO,
        stylo::Size::FitContentFunction(_) => taffy::Dimension::AUTO,
        stylo::Size::Stretch => taffy::Dimension::AUTO,

        // Anchor positioning will be flagged off for time being
        stylo::Size::AnchorSizeFunction(_) => unreachable!(),
        stylo::Size::AnchorContainingCalcFunction(_) => unreachable!(),
    }
}

#[inline]
pub fn max_size_dimension(val: &stylo::MaxSize) -> taffy::Dimension {
    match val {
        stylo::MaxSize::LengthPercentage(val) => length_percentage(&val.0).into(),
        stylo::MaxSize::None => taffy::Dimension::AUTO,

        // TODO: implement other values in Taffy
        stylo::MaxSize::MaxContent => taffy::Dimension::AUTO,
        stylo::MaxSize::MinContent => taffy::Dimension::AUTO,
        stylo::MaxSize::FitContent => taffy::Dimension::AUTO,
        stylo::MaxSize::FitContentFunction(_) => taffy::Dimension::AUTO,
        stylo::MaxSize::Stretch => taffy::Dimension::AUTO,

        // Anchor positioning will be flagged off for time being
        stylo::MaxSize::AnchorSizeFunction(_) => unreachable!(),
        stylo::MaxSize::AnchorContainingCalcFunction(_) => unreachable!(),
    }
}

#[inline]
pub fn margin(val: &stylo::MarginVal) -> taffy::LengthPercentageAuto {
    match val {
        stylo::MarginVal::Auto => taffy::LengthPercentageAuto::AUTO,
        stylo::MarginVal::LengthPercentage(val) => length_percentage(val).into(),

        // Anchor positioning will be flagged off for time being
        stylo::MarginVal::AnchorSizeFunction(_) => unreachable!(),
        stylo::MarginVal::AnchorContainingCalcFunction(_) => unreachable!(),
    }
}

#[inline]
pub fn inset(val: &stylo::InsetVal) -> taffy::LengthPercentageAuto {
    match val {
        stylo::InsetVal::Auto => taffy::LengthPercentageAuto::AUTO,
        stylo::InsetVal::LengthPercentage(val) => length_percentage(val).into(),

        // Anchor positioning will be flagged off for time being
        stylo::InsetVal::AnchorSizeFunction(_) => unreachable!(),
        stylo::InsetVal::AnchorFunction(_) => unreachable!(),
        stylo::InsetVal::AnchorContainingCalcFunction(_) => unreachable!(),
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
pub fn position(input: stylo::Position) -> taffy::Position {
    match input {
        // TODO: support position:static
        stylo::Position::Relative => taffy::Position::Relative,
        stylo::Position::Static => taffy::Position::Relative,

        // TODO: support position:fixed and sticky
        stylo::Position::Absolute => taffy::Position::Absolute,
        stylo::Position::Fixed => taffy::Position::Absolute,
        stylo::Position::Sticky => taffy::Position::Relative,
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
pub fn content_alignment(input: stylo::ContentDistribution) -> Option<taffy::AlignContent> {
    match input.primary().value() {
        stylo::AlignFlags::NORMAL => None,
        stylo::AlignFlags::AUTO => None,
        stylo::AlignFlags::START => Some(taffy::AlignContent::Start),
        stylo::AlignFlags::END => Some(taffy::AlignContent::End),
        stylo::AlignFlags::LEFT => Some(taffy::AlignContent::Start),
        stylo::AlignFlags::RIGHT => Some(taffy::AlignContent::End),
        stylo::AlignFlags::FLEX_START => Some(taffy::AlignContent::FlexStart),
        stylo::AlignFlags::STRETCH => Some(taffy::AlignContent::Stretch),
        stylo::AlignFlags::FLEX_END => Some(taffy::AlignContent::FlexEnd),
        stylo::AlignFlags::CENTER => Some(taffy::AlignContent::Center),
        stylo::AlignFlags::SPACE_BETWEEN => Some(taffy::AlignContent::SpaceBetween),
        stylo::AlignFlags::SPACE_AROUND => Some(taffy::AlignContent::SpaceAround),
        stylo::AlignFlags::SPACE_EVENLY => Some(taffy::AlignContent::SpaceEvenly),
        // Should never be hit. But no real reason to panic here.
        _ => None,
    }
}

#[inline]
pub fn item_alignment(input: stylo::AlignFlags) -> Option<taffy::AlignItems> {
    match input.value() {
        stylo::AlignFlags::AUTO => None,
        stylo::AlignFlags::NORMAL => Some(taffy::AlignItems::Stretch),
        stylo::AlignFlags::STRETCH => Some(taffy::AlignItems::Stretch),
        stylo::AlignFlags::FLEX_START => Some(taffy::AlignItems::FlexStart),
        stylo::AlignFlags::FLEX_END => Some(taffy::AlignItems::FlexEnd),
        stylo::AlignFlags::SELF_START => Some(taffy::AlignItems::Start),
        stylo::AlignFlags::SELF_END => Some(taffy::AlignItems::End),
        stylo::AlignFlags::START => Some(taffy::AlignItems::Start),
        stylo::AlignFlags::END => Some(taffy::AlignItems::End),
        stylo::AlignFlags::LEFT => Some(taffy::AlignItems::Start),
        stylo::AlignFlags::RIGHT => Some(taffy::AlignItems::End),
        stylo::AlignFlags::CENTER => Some(taffy::AlignItems::Center),
        stylo::AlignFlags::BASELINE => Some(taffy::AlignItems::Baseline),
        // Should never be hit. But no real reason to panic here.
        _ => None,
    }
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
pub fn grid_line(input: &stylo::GridLine) -> taffy::GridPlacement {
    if input.is_auto() {
        taffy::GridPlacement::Auto
    } else if input.is_span {
        taffy::style_helpers::span(input.line_num.try_into().unwrap())
    } else if input.line_num == 0 {
        taffy::GridPlacement::Auto
    } else {
        taffy::style_helpers::line(input.line_num.try_into().unwrap())
    }
}

#[inline]
pub fn grid_template_tracks(
    input: &stylo::GridTemplateComponent,
) -> Vec<taffy::TrackSizingFunction> {
    match input {
        stylo::GenericGridTemplateComponent::None => Vec::new(),
        stylo::GenericGridTemplateComponent::TrackList(list) => list
            .values
            .iter()
            .map(|track| match track {
                stylo::TrackListValue::TrackSize(size) => {
                    taffy::TrackSizingFunction::Single(track_size(size))
                },
                stylo::TrackListValue::TrackRepeat(repeat) => taffy::TrackSizingFunction::Repeat(
                    track_repeat(repeat.count),
                    repeat.track_sizes.iter().map(track_size).collect(),
                ),
            })
            .collect(),

        // TODO: Implement subgrid and masonry
        stylo::GenericGridTemplateComponent::Subgrid(_) => Vec::new(),
        stylo::GenericGridTemplateComponent::Masonry => Vec::new(),
    }
}

#[inline]
pub fn grid_auto_tracks(
    input: &stylo::ImplicitGridTracks,
) -> Vec<taffy::NonRepeatedTrackSizingFunction> {
    input.0.iter().map(track_size).collect()
}

#[inline]
pub fn track_repeat(input: stylo::RepeatCount<i32>) -> taffy::GridTrackRepetition {
    match input {
        stylo::RepeatCount::Number(val) => {
            taffy::GridTrackRepetition::Count(val.try_into().unwrap())
        },
        stylo::RepeatCount::AutoFill => taffy::GridTrackRepetition::AutoFill,
        stylo::RepeatCount::AutoFit => taffy::GridTrackRepetition::AutoFit,
    }
}

#[inline]
pub fn track_size(
    input: &stylo::TrackSize<stylo::LengthPercentage>,
) -> taffy::NonRepeatedTrackSizingFunction {
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
                stylo::TrackBreadth::Fr(_) => unreachable!(),
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
        stylo::TrackBreadth::Fr(_) => taffy::MinTrackSizingFunction::AUTO,
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
        stylo::TrackBreadth::Fr(val) => fr(*val),
        stylo::TrackBreadth::Auto => taffy::MaxTrackSizingFunction::AUTO,
        stylo::TrackBreadth::MinContent => taffy::MaxTrackSizingFunction::MIN_CONTENT,
        stylo::TrackBreadth::MaxContent => taffy::MaxTrackSizingFunction::MAX_CONTENT,
    }
}
