/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ops::Deref;
use style::properties::ComputedValues;

/// A wrapper struct for anything that Deref's to a [`stylo::ComputedValues`], which implements Taffy's layout traits 
/// and can used with Taffy's layout algorithms.
pub struct TaffyStyloStyle<T: Deref<Target = ComputedValues>>(pub T);

impl<T: Deref<Target = ComputedValues>> From<T> for TaffyStyloStyle<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::CoreStyle for TaffyStyloStyle<T> {
    #[inline]
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        convert::box_generation_mode(self.0.get_box().display)
    }

    #[inline]
    fn is_block(&self) -> bool {
        convert::is_block(self.0.get_box().display)
    }

    #[inline]
    fn box_sizing(&self) -> taffy::BoxSizing {
        convert::box_sizing(self.0.get_position().box_sizing)
    }

    #[inline]
    fn overflow(&self) -> taffy::Point<taffy::Overflow> {
        let box_styles = self.0.get_box();
        taffy::Point {
            x: convert::overflow(box_styles.overflow_x),
            y: convert::overflow(box_styles.overflow_y),
        }
    }

    #[inline]
    fn scrollbar_width(&self) -> f32 {
        0.0
    }

    #[inline]
    fn position(&self) -> taffy::Position {
        convert::position(self.0.get_box().position)
    }

    #[inline]
    fn inset(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        let position_styles = self.0.get_position();
        taffy::Rect {
            left: convert::inset(&position_styles.left),
            right: convert::inset(&position_styles.right),
            top: convert::inset(&position_styles.top),
            bottom: convert::inset(&position_styles.bottom),
        }
    }

    #[inline]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.0.get_position();
        taffy::Size {
            width: convert::dimension(&position_styles.width),
            height: convert::dimension(&position_styles.height),
        }
    }

    #[inline]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.0.get_position();
        taffy::Size {
            width: convert::dimension(&position_styles.min_width),
            height: convert::dimension(&position_styles.min_height),
        }
    }

    #[inline]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        let position_styles = self.0.get_position();
        taffy::Size {
            width: convert::max_size_dimension(&position_styles.max_width),
            height: convert::max_size_dimension(&position_styles.max_height),
        }
    }

    #[inline]
    fn aspect_ratio(&self) -> Option<f32> {
        convert::aspect_ratio(self.0.get_position().aspect_ratio)
    }

    #[inline]
    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        let margin_styles = self.0.get_margin();
        taffy::Rect {
            left: convert::margin(&margin_styles.margin_left),
            right: convert::margin(&margin_styles.margin_right),
            top: convert::margin(&margin_styles.margin_top),
            bottom: convert::margin(&margin_styles.margin_bottom),
        }
    }

    #[inline]
    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        let padding_styles = self.0.get_padding();
        taffy::Rect {
            left: convert::length_percentage(&padding_styles.padding_left.0),
            right: convert::length_percentage(&padding_styles.padding_right.0),
            top: convert::length_percentage(&padding_styles.padding_top.0),
            bottom: convert::length_percentage(&padding_styles.padding_bottom.0),
        }
    }

    #[inline]
    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        let border_styles = self.0.get_border();
        taffy::Rect {
            left: taffy::LengthPercentage::Length(border_styles.border_left_width.to_f32_px()),
            right: taffy::LengthPercentage::Length(border_styles.border_right_width.to_f32_px()),
            top: taffy::LengthPercentage::Length(border_styles.border_top_width.to_f32_px()),
            bottom: taffy::LengthPercentage::Length(border_styles.border_bottom_width.to_f32_px()),
        }
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::FlexboxContainerStyle for TaffyStyloStyle<T> {
    #[inline]
    fn flex_direction(&self) -> taffy::FlexDirection {
        convert::flex_direction(self.0.get_position().flex_direction)
    }

    #[inline]
    fn flex_wrap(&self) -> taffy::FlexWrap {
        convert::flex_wrap(self.0.get_position().flex_wrap)
    }

    #[inline]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        let position_styles = self.0.get_position();
        taffy::Size {
            width: convert::gap(&position_styles.column_gap),
            height: convert::gap(&position_styles.row_gap),
        }
    }

    #[inline]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        convert::content_alignment(self.0.get_position().align_content.0)
    }

    #[inline]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.0.get_position().align_items.0)
    }

    #[inline]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        convert::content_alignment(self.0.get_position().justify_content.0)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::FlexboxItemStyle for TaffyStyloStyle<T> {
    #[inline]
    fn flex_basis(&self) -> taffy::Dimension {
        convert::flex_basis(&self.0.get_position().flex_basis)
    }

    #[inline]
    fn flex_grow(&self) -> f32 {
        self.0.get_position().flex_grow.0
    }

    #[inline]
    fn flex_shrink(&self) -> f32 {
        self.0.get_position().flex_shrink.0
    }

    #[inline]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.0.get_position().align_self.0 .0)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridContainerStyle for TaffyStyloStyle<T> {
    type TemplateTrackList<'a> = Vec<taffy::TrackSizingFunction> where Self: 'a;
    type AutoTrackList<'a> = Vec<taffy::NonRepeatedTrackSizingFunction> where Self: 'a;

    #[inline]
    fn grid_template_rows(&self) -> Self::TemplateTrackList<'_> {
        convert::grid_template_tracks(&self.0.get_position().grid_template_rows)
    }

    #[inline]
    fn grid_template_columns(&self) -> Self::TemplateTrackList<'_> {
        convert::grid_template_tracks(&self.0.get_position().grid_template_columns)
    }

    #[inline]
    fn grid_auto_rows(&self) -> Self::AutoTrackList<'_> {
        convert::grid_auto_tracks(&self.0.get_position().grid_auto_rows)
    }

    #[inline]
    fn grid_auto_columns(&self) -> Self::AutoTrackList<'_> {
        convert::grid_auto_tracks(&self.0.get_position().grid_auto_columns)
    }

    #[inline]
    fn grid_auto_flow(&self) -> taffy::GridAutoFlow {
        convert::grid_auto_flow(self.0.get_position().grid_auto_flow)
    }

    #[inline]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        let position_styles = self.0.get_position();
        taffy::Size {
            width: convert::gap(&position_styles.column_gap),
            height: convert::gap(&position_styles.row_gap),
        }
    }

    #[inline]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        convert::content_alignment(self.0.get_position().align_content.0)
    }

    #[inline]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        convert::content_alignment(self.0.get_position().justify_content.0)
    }

    #[inline]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.0.get_position().align_items.0)
    }

    #[inline]
    fn justify_items(&self) -> Option<taffy::AlignItems> {
        convert::item_alignment(self.0.get_position().justify_items.computed.0)
    }
}

impl<T: Deref<Target = ComputedValues>> taffy::GridItemStyle for TaffyStyloStyle<T> {
    #[inline]
    fn grid_row(&self) -> taffy::Line<taffy::GridPlacement> {
        let position_styles = self.0.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_row_start),
            end: convert::grid_line(&position_styles.grid_row_end),
        }
    }

    #[inline]
    fn grid_column(&self) -> taffy::Line<taffy::GridPlacement> {
        let position_styles = self.0.get_position();
        taffy::Line {
            start: convert::grid_line(&position_styles.grid_column_start),
            end: convert::grid_line(&position_styles.grid_column_end),
        }
    }

    #[inline]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.0.get_position().align_self.0 .0)
    }

    #[inline]
    fn justify_self(&self) -> Option<taffy::AlignSelf> {
        convert::item_alignment(self.0.get_position().justify_self.0 .0)
    }
}

// Module of type aliases so we can refer to stylo types with nicer names
mod stylo {
    pub(crate) use style::computed_values::flex_direction::T as FlexDirection;
    pub(crate) use style::computed_values::flex_wrap::T as FlexWrap;
    pub(crate) use style::computed_values::grid_auto_flow::T as GridAutoFlow;
    pub(crate) use style::properties::generated::longhands::box_sizing::computed_value::T as BoxSizing;
    pub(crate) use style::properties::longhands::aspect_ratio::computed_value::T as AspectRatio;
    pub(crate) use style::properties::longhands::position::computed_value::T as Position;
    use style::values::computed::Percentage;
    pub(crate) use style::values::computed::{
        GridLine, GridTemplateComponent, ImplicitGridTracks, LengthPercentage,
    };
    pub(crate) use style::values::generics::flex::GenericFlexBasis;
    pub(crate) use style::values::generics::grid::{
        RepeatCount, TrackBreadth, TrackListValue, TrackSize,
    };
    use style::values::generics::length::GenericMargin;
    pub(crate) use style::values::generics::length::{
        GenericLengthPercentageOrNormal, GenericMaxSize, GenericSize,
    };
    use style::values::generics::position::Inset as GenericInset;
    pub(crate) use style::values::generics::position::PreferredRatio;
    pub(crate) use style::values::generics::NonNegative;
    pub(crate) use style::values::specified::align::{AlignFlags, ContentDistribution};
    pub(crate) use style::values::specified::box_::{
        Display, DisplayInside, DisplayOutside, Overflow,
    };
    pub(crate) use style::values::specified::GenericGridTemplateComponent;
    pub(crate) type MarginVal = GenericMargin<LengthPercentage>;
    pub(crate) type InsetVal = GenericInset<Percentage, LengthPercentage>;
    pub(crate) type Size = GenericSize<NonNegative<LengthPercentage>>;
    pub(crate) type MaxSize = GenericMaxSize<NonNegative<LengthPercentage>>;
    pub(crate) type FlexBasis = GenericFlexBasis<Size>;
    pub(crate) type Gap = GenericLengthPercentageOrNormal<NonNegative<LengthPercentage>>;
}

mod convert {
  use super::stylo;

  #[inline]
  pub(crate) fn length_percentage(val: &stylo::LengthPercentage) -> taffy::LengthPercentage {
      if let Some(length) = val.to_length() {
          taffy::LengthPercentage::Length(length.px())
      } else if let Some(val) = val.to_percentage() {
          taffy::LengthPercentage::Percent(val.0)
      } else {
          // TODO: Support calc
          taffy::LengthPercentage::Percent(0.0)
      }
  }

  #[inline]
  pub(crate) fn margin(val: &stylo::MarginVal) -> taffy::LengthPercentageAuto {
      match val {
          stylo::MarginVal::Auto => taffy::LengthPercentageAuto::Auto,
          stylo::MarginVal::LengthPercentage(val) => length_percentage(val).into(),

          // Anchor positioning will be flagged off for time being
          stylo::MarginVal::AnchorSizeFunction(_) => unreachable!(),
      }
  }

  #[inline]
  pub(crate) fn inset(val: &stylo::InsetVal) -> taffy::LengthPercentageAuto {
      match val {
          stylo::InsetVal::Auto => taffy::LengthPercentageAuto::Auto,
          stylo::InsetVal::LengthPercentage(val) => length_percentage(val).into(),

          // Anchor positioning will be flagged off for time being
          stylo::InsetVal::AnchorSizeFunction(_) => unreachable!(),
          stylo::InsetVal::AnchorFunction(_) => unreachable!(),
      }
  }

  #[inline]
  pub(crate) fn dimension(val: &stylo::Size) -> taffy::Dimension {
      match val {
          stylo::Size::LengthPercentage(val) => length_percentage(&val.0).into(),
          stylo::Size::Auto => taffy::Dimension::Auto,

          // TODO: implement other values in Taffy
          stylo::Size::MaxContent => taffy::Dimension::Auto,
          stylo::Size::MinContent => taffy::Dimension::Auto,
          stylo::Size::FitContent => taffy::Dimension::Auto,
          stylo::Size::Stretch => taffy::Dimension::Auto,

          // Anchor positioning will be flagged off for time being
          stylo::Size::AnchorSizeFunction(_) => unreachable!(),
      }
  }

  #[inline]
  pub(crate) fn max_size_dimension(val: &stylo::MaxSize) -> taffy::Dimension {
      match val {
          stylo::MaxSize::LengthPercentage(val) => length_percentage(&val.0).into(),
          stylo::MaxSize::None => taffy::Dimension::Auto,

          // TODO: implement other values in Taffy
          stylo::MaxSize::MaxContent => taffy::Dimension::Auto,
          stylo::MaxSize::MinContent => taffy::Dimension::Auto,
          stylo::MaxSize::FitContent => taffy::Dimension::Auto,
          stylo::MaxSize::Stretch => taffy::Dimension::Auto,

          // Anchor positioning will be flagged off for time being
          stylo::MaxSize::AnchorSizeFunction(_) => unreachable!(),
      }
  }

  #[inline]
  pub(crate) fn is_block(input: stylo::Display) -> bool {
      matches!(input.outside(), stylo::DisplayOutside::Block)
          && matches!(
              input.inside(),
              stylo::DisplayInside::Flow | stylo::DisplayInside::FlowRoot
          )
  }

  #[inline]
  pub(crate) fn box_generation_mode(input: stylo::Display) -> taffy::BoxGenerationMode {
      match input.inside() {
          stylo::DisplayInside::None => taffy::BoxGenerationMode::None,
          // stylo::DisplayInside::Contents => display = taffy::BoxGenerationMode::Contents,
          _ => taffy::BoxGenerationMode::Normal,
      }
  }

  #[inline]
  pub(crate) fn box_sizing(input: stylo::BoxSizing) -> taffy::BoxSizing {
      match input {
          stylo::BoxSizing::BorderBox => taffy::BoxSizing::BorderBox,
          stylo::BoxSizing::ContentBox => taffy::BoxSizing::ContentBox,
      }
  }

  #[inline]
  pub(crate) fn position(input: stylo::Position) -> taffy::Position {
      match input {
          // TODO: support position:static
          stylo::Position::Relative => taffy::Position::Relative,
          stylo::Position::Static => taffy::Position::Relative,

          // TODO: support position:fixed and sticky
          stylo::Position::Absolute => taffy::Position::Absolute,
          stylo::Position::Fixed => taffy::Position::Absolute,
          stylo::Position::Sticky => taffy::Position::Absolute,
      }
  }

  #[inline]
  pub(crate) fn overflow(input: stylo::Overflow) -> taffy::Overflow {
      // TODO: Enable Overflow::Clip in servo configuration of stylo
      match input {
          stylo::Overflow::Visible => taffy::Overflow::Visible,
          stylo::Overflow::Hidden => taffy::Overflow::Hidden,
          stylo::Overflow::Scroll => taffy::Overflow::Scroll,
          // TODO: Support Overflow::Auto in Taffy
          stylo::Overflow::Auto => taffy::Overflow::Scroll,
      }
  }

  #[inline]
  pub(crate) fn aspect_ratio(input: stylo::AspectRatio) -> Option<f32> {
      match input.ratio {
          stylo::PreferredRatio::None => None,
          stylo::PreferredRatio::Ratio(val) => Some(val.0 .0 / val.1 .0),
      }
  }

  #[inline]
  pub(crate) fn gap(input: &stylo::Gap) -> taffy::LengthPercentage {
      match input {
          // For Flexbox and CSS Grid the "normal" value is 0px. This may need to be updated
          // if we ever implement multi-column layout.
          stylo::Gap::Normal => taffy::LengthPercentage::Length(0.0),
          stylo::Gap::LengthPercentage(val) => length_percentage(&val.0),
      }
  }

  #[inline]
  pub(crate) fn flex_basis(input: &stylo::FlexBasis) -> taffy::Dimension {
      // TODO: Support flex-basis: content in Taffy
      match input {
          stylo::FlexBasis::Content => taffy::Dimension::Auto,
          stylo::FlexBasis::Size(size) => dimension(size),
      }
  }

  #[inline]
  pub(crate) fn flex_direction(input: stylo::FlexDirection) -> taffy::FlexDirection {
      match input {
          stylo::FlexDirection::Row => taffy::FlexDirection::Row,
          stylo::FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
          stylo::FlexDirection::Column => taffy::FlexDirection::Column,
          stylo::FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
      }
  }

  #[inline]
  pub(crate) fn flex_wrap(input: stylo::FlexWrap) -> taffy::FlexWrap {
      match input {
          stylo::FlexWrap::Wrap => taffy::FlexWrap::Wrap,
          stylo::FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
          stylo::FlexWrap::Nowrap => taffy::FlexWrap::NoWrap,
      }
  }

  #[inline]
  pub(crate) fn content_alignment(input: stylo::ContentDistribution) -> Option<taffy::AlignContent> {
      match input.primary().value() {
          stylo::AlignFlags::NORMAL => None,
          stylo::AlignFlags::AUTO => None,
          stylo::AlignFlags::START => Some(taffy::AlignContent::Start),
          stylo::AlignFlags::END => Some(taffy::AlignContent::End),
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
  pub(crate) fn item_alignment(input: stylo::AlignFlags) -> Option<taffy::AlignItems> {
      match input.value() {
          stylo::AlignFlags::NORMAL => None,
          stylo::AlignFlags::AUTO => None,
          stylo::AlignFlags::STRETCH => Some(taffy::AlignItems::Stretch),
          stylo::AlignFlags::FLEX_START => Some(taffy::AlignItems::FlexStart),
          stylo::AlignFlags::FLEX_END => Some(taffy::AlignItems::FlexEnd),
          stylo::AlignFlags::START => Some(taffy::AlignItems::Start),
          stylo::AlignFlags::END => Some(taffy::AlignItems::End),
          stylo::AlignFlags::CENTER => Some(taffy::AlignItems::Center),
          stylo::AlignFlags::BASELINE => Some(taffy::AlignItems::Baseline),
          // Should never be hit. But no real reason to panic here.
          _ => None,
      }
  }

  #[inline]
  pub(crate) fn grid_auto_flow(input: stylo::GridAutoFlow) -> taffy::GridAutoFlow {
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
  pub(crate) fn grid_line(input: &stylo::GridLine) -> taffy::GridPlacement {
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
  pub(crate) fn grid_template_tracks(
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
  pub(crate) fn grid_auto_tracks(
      input: &stylo::ImplicitGridTracks,
  ) -> Vec<taffy::NonRepeatedTrackSizingFunction> {
      input.0.iter().map(track_size).collect()
  }

  #[inline]
  pub(crate) fn track_repeat(input: stylo::RepeatCount<i32>) -> taffy::GridTrackRepetition {
      match input {
          stylo::RepeatCount::Number(val) => {
              taffy::GridTrackRepetition::Count(val.try_into().unwrap())
          },
          stylo::RepeatCount::AutoFill => taffy::GridTrackRepetition::AutoFill,
          stylo::RepeatCount::AutoFit => taffy::GridTrackRepetition::AutoFit,
      }
  }

  #[inline]
  pub(crate) fn track_size(
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
              min: taffy::MinTrackSizingFunction::Auto,
              max: taffy::MaxTrackSizingFunction::FitContent(match limit {
                  stylo::TrackBreadth::Breadth(lp) => length_percentage(lp),

                  // Are these valid? Taffy doesn't support this in any case
                  stylo::TrackBreadth::Fr(_) => unreachable!(),
                  stylo::TrackBreadth::Auto => unreachable!(),
                  stylo::TrackBreadth::MinContent => unreachable!(),
                  stylo::TrackBreadth::MaxContent => unreachable!(),
              }),
          },
      }
  }

  #[inline]
  pub(crate) fn min_track(
      input: &stylo::TrackBreadth<stylo::LengthPercentage>,
  ) -> taffy::MinTrackSizingFunction {
      match input {
          stylo::TrackBreadth::Breadth(lp) => {
              taffy::MinTrackSizingFunction::Fixed(length_percentage(lp))
          },
          stylo::TrackBreadth::Fr(_) => taffy::MinTrackSizingFunction::Auto,
          stylo::TrackBreadth::Auto => taffy::MinTrackSizingFunction::Auto,
          stylo::TrackBreadth::MinContent => taffy::MinTrackSizingFunction::MinContent,
          stylo::TrackBreadth::MaxContent => taffy::MinTrackSizingFunction::MaxContent,
      }
  }

  #[inline]
  pub(crate) fn max_track(
      input: &stylo::TrackBreadth<stylo::LengthPercentage>,
  ) -> taffy::MaxTrackSizingFunction {
      match input {
          stylo::TrackBreadth::Breadth(lp) => {
              taffy::MaxTrackSizingFunction::Fixed(length_percentage(lp))
          },
          stylo::TrackBreadth::Fr(val) => taffy::MaxTrackSizingFunction::Fraction(*val),
          stylo::TrackBreadth::Auto => taffy::MaxTrackSizingFunction::Auto,
          stylo::TrackBreadth::MinContent => taffy::MaxTrackSizingFunction::MinContent,
          stylo::TrackBreadth::MaxContent => taffy::MaxTrackSizingFunction::MaxContent,
      }
  }
}

