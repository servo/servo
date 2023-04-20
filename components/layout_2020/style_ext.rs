/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::geom::flow_relative;
use crate::ContainingBlock;
use style::computed_values::mix_blend_mode::T as ComputedMixBlendMode;
use style::computed_values::position::T as ComputedPosition;
use style::computed_values::transform_style::T as ComputedTransformStyle;
use style::properties::longhands::backface_visibility::computed_value::T as BackfaceVisiblity;
use style::properties::longhands::box_sizing::computed_value::T as BoxSizing;
use style::properties::ComputedValues;
use style::values::computed::image::Image as ComputedImageLayer;
use style::values::computed::{
    Length, LengthOrAuto, LengthPercentage, LengthPercentageOrAuto, NonNegativeLengthPercentage,
    Size,
};
use style::values::generics::box_::Perspective;
use style::values::generics::length::MaxSize;
use style::values::specified::box_ as stylo;
use style::Zero;
use webrender_api as wr;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Display {
    None,
    Contents,
    GeneratingBox(DisplayGeneratingBox),
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayGeneratingBox {
    OutsideInside {
        outside: DisplayOutside,
        inside: DisplayInside,
    },
    // Layout-internal display types go here:
    // https://drafts.csswg.org/css-display-3/#layout-specific-display
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayOutside {
    Block,
    Inline,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum DisplayInside {
    // “list-items are limited to the Flow Layout display types”
    // https://drafts.csswg.org/css-display/#list-items
    Flow { is_list_item: bool },
    FlowRoot { is_list_item: bool },
    Flex,
}

/// Percentages resolved but not `auto` margins
pub(crate) struct PaddingBorderMargin {
    pub padding: flow_relative::Sides<Length>,
    pub border: flow_relative::Sides<Length>,
    pub margin: flow_relative::Sides<LengthOrAuto>,

    /// Pre-computed sums in each axis
    pub padding_border_sums: flow_relative::Vec2<Length>,
}

pub(crate) trait ComputedValuesExt {
    fn inline_size_is_length(&self) -> bool;
    fn inline_box_offsets_are_both_non_auto(&self) -> bool;
    fn min_box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto>;
    fn max_box_size(&self) -> flow_relative::Vec2<Option<&LengthPercentage>>;
    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto>;
    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto>;
    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<Option<Length>>;
    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin;
    fn has_transform_or_perspective(&self) -> bool;
    fn effective_z_index(&self) -> i32;
    fn establishes_stacking_context(&self) -> bool;
    fn establishes_containing_block(&self) -> bool;
    fn establishes_containing_block_for_all_descendants(&self) -> bool;
    fn background_is_transparent(&self) -> bool;
    fn get_webrender_primitive_flags(&self) -> wr::PrimitiveFlags;
}

impl ComputedValuesExt for ComputedValues {
    fn inline_size_is_length(&self) -> bool {
        matches!(self.content_inline_size(), Size::LengthPercentage(lp) if lp.0.to_length().is_some())
    }

    fn inline_box_offsets_are_both_non_auto(&self) -> bool {
        let position = self.logical_position();
        !position.inline_start.is_auto() && !position.inline_end.is_auto()
    }

    fn min_box_size(&self) -> flow_relative::Vec2<LengthPercentageOrAuto> {
        flow_relative::Vec2 {
            inline: size_to_length(self.min_inline_size()),
            block: size_to_length(self.min_block_size()),
        }
    }

    fn max_box_size(&self) -> flow_relative::Vec2<Option<&LengthPercentage>> {
        fn unwrap(max_size: &MaxSize<NonNegativeLengthPercentage>) -> Option<&LengthPercentage> {
            match max_size {
                MaxSize::LengthPercentage(length) => Some(&length.0),
                MaxSize::None => None,
            }
        }
        flow_relative::Vec2 {
            inline: unwrap(&self.max_inline_size()),
            block: unwrap(&self.max_block_size()),
        }
    }

    fn content_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto> {
        let box_size = flow_relative::Vec2 {
            inline: size_to_length(self.content_inline_size())
                .percentage_relative_to(containing_block.inline_size),
            block: size_to_length(self.content_block_size())
                .maybe_percentage_relative_to(containing_block.block_size.non_auto()),
        };

        match self.get_position().box_sizing {
            BoxSizing::ContentBox => box_size,
            BoxSizing::BorderBox => flow_relative::Vec2 {
                // These may be negative, but will later be clamped by `min-width`/`min-height`
                // which is clamped to zero.
                inline: box_size.inline.map(|i| i - pbm.padding_border_sums.inline),
                block: box_size.block.map(|b| b - pbm.padding_border_sums.block),
            },
        }
    }

    fn content_min_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<LengthOrAuto> {
        let min_box_size = self
            .min_box_size()
            .percentages_relative_to(containing_block);
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => min_box_size,
            BoxSizing::BorderBox => flow_relative::Vec2 {
                // Clamp to zero to make sure the used size components are non-negative
                inline: min_box_size
                    .inline
                    .map(|i| (i - pbm.padding_border_sums.inline).max(Length::zero())),
                block: min_box_size
                    .block
                    .map(|b| (b - pbm.padding_border_sums.block).max(Length::zero())),
            },
        }
    }

    fn content_max_box_size(
        &self,
        containing_block: &ContainingBlock,
        pbm: &PaddingBorderMargin,
    ) -> flow_relative::Vec2<Option<Length>> {
        let max_box_size = self
            .max_box_size()
            .percentages_relative_to(containing_block);
        match self.get_position().box_sizing {
            BoxSizing::ContentBox => max_box_size,
            BoxSizing::BorderBox => {
                // This may be negative, but will later be clamped by `min-width`
                // which itself is clamped to zero.
                flow_relative::Vec2 {
                    inline: max_box_size
                        .inline
                        .map(|i| i - pbm.padding_border_sums.inline),
                    block: max_box_size
                        .block
                        .map(|b| b - pbm.padding_border_sums.block),
                }
            },
        }
    }

    fn padding_border_margin(&self, containing_block: &ContainingBlock) -> PaddingBorderMargin {
        let padding = self
            .logical_padding()
            .map(|v| v.percentage_relative_to(containing_block.inline_size));
        let margin = self
            .logical_margin()
            .map(|v| v.percentage_relative_to(containing_block.inline_size));
        let border = self.logical_border_width();
        PaddingBorderMargin {
            padding_border_sums: flow_relative::Vec2 {
                inline: padding.inline_start_end() + border.inline_start_end(),
                block: padding.block_start_end() + border.block_start_end(),
            },
            padding: padding.into(),
            border: border.into(),
            margin: margin.into(),
        }
    }

    /// Returns true if this style has a transform, or perspective property set.
    fn has_transform_or_perspective(&self) -> bool {
        !self.get_box().transform.0.is_empty() || self.get_box().perspective != Perspective::None
    }

    /// Get the effective z-index of this fragment. Z-indices only apply to positioned elements
    /// per CSS 2 9.9.1 (http://www.w3.org/TR/CSS2/visuren.html#z-index), so this value may differ
    /// from the value specified in the style.
    fn effective_z_index(&self) -> i32 {
        match self.get_box().position {
            ComputedPosition::Static => 0,
            _ => self.get_position().z_index.integer_or(0),
        }
    }

    /// Returns true if this fragment establishes a new stacking context and false otherwise.
    fn establishes_stacking_context(&self) -> bool {
        let effects = self.get_effects();
        if effects.opacity != 1.0 {
            return true;
        }

        if effects.mix_blend_mode != ComputedMixBlendMode::Normal {
            return true;
        }

        if self.has_transform_or_perspective() {
            return true;
        }

        if !self.get_effects().filter.0.is_empty() {
            return true;
        }

        if self.get_box().transform_style == ComputedTransformStyle::Preserve3d ||
            self.overrides_transform_style()
        {
            return true;
        }

        // Fixed position and sticky position always create stacking contexts.
        // TODO(mrobinson): We need to handle sticky positioning here when we support it.
        if self.get_box().position == ComputedPosition::Fixed {
            return true;
        }

        // Statically positioned fragments don't establish stacking contexts if the previous
        // conditions are not fulfilled. Furthermore, z-index doesn't apply to statically
        // positioned fragments.
        if self.get_box().position == ComputedPosition::Static {
            return false;
        }

        // For absolutely and relatively positioned fragments we only establish a stacking
        // context if there is a z-index set.
        // See https://www.w3.org/TR/CSS2/visuren.html#z-index
        !self.get_position().z_index.is_auto()
    }

    fn establishes_containing_block(&self) -> bool {
        if self.establishes_containing_block_for_all_descendants() {
            return true;
        }

        self.clone_position() != ComputedPosition::Static
    }

    /// Returns true if this style establishes a containing block for all descendants
    /// including fixed and absolutely positioned ones.
    fn establishes_containing_block_for_all_descendants(&self) -> bool {
        if self.get_box().display.outside() != stylo::DisplayOutside::Inline &&
            self.has_transform_or_perspective()
        {
            return true;
        }

        if !self.get_effects().filter.0.is_empty() {
            return true;
        }

        // TODO: We need to handle CSS Contain here.
        false
    }

    /// Whether or not this style specifies a non-transparent background.
    fn background_is_transparent(&self) -> bool {
        let background = self.get_background();
        let color = self.resolve_color(background.background_color);
        color.alpha == 0 &&
            background
                .background_image
                .0
                .iter()
                .all(|layer| matches!(layer, ComputedImageLayer::None))
    }

    /// Generate appropriate WebRender `PrimitiveFlags` that should be used
    /// for display items generated by the `Fragment` which owns this style.
    fn get_webrender_primitive_flags(&self) -> wr::PrimitiveFlags {
        match self.get_box().backface_visibility {
            BackfaceVisiblity::Visible => wr::PrimitiveFlags::default(),
            BackfaceVisiblity::Hidden => wr::PrimitiveFlags::empty(),
        }
    }
}

impl From<stylo::Display> for Display {
    fn from(packed: stylo::Display) -> Self {
        let inside = match packed.inside() {
            stylo::DisplayInside::Flow => DisplayInside::Flow {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::FlowRoot => DisplayInside::FlowRoot {
                is_list_item: packed.is_list_item(),
            },
            stylo::DisplayInside::Flex => DisplayInside::Flex,

            // These should not be values of DisplayInside, but oh well
            stylo::DisplayInside::None => return Display::None,
            stylo::DisplayInside::Contents => return Display::Contents,
        };
        let outside = match packed.outside() {
            stylo::DisplayOutside::Block => DisplayOutside::Block,
            stylo::DisplayOutside::Inline => DisplayOutside::Inline,

            // This should not be a value of DisplayInside, but oh well
            stylo::DisplayOutside::None => return Display::None,
        };
        Display::GeneratingBox(DisplayGeneratingBox::OutsideInside { outside, inside })
    }
}

fn size_to_length(size: &Size) -> LengthPercentageOrAuto {
    match size {
        Size::LengthPercentage(length) => {
            LengthPercentageOrAuto::LengthPercentage(length.0.clone())
        },
        Size::Auto => LengthPercentageOrAuto::Auto,
    }
}
