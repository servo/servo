/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom_traversal::NodeExt;
use crate::fragments::{Fragment, ImageFragment};
use crate::geom::{flow_relative, physical};
use crate::style_ext::ComputedValuesExt;
use crate::ContainingBlock;
use net_traits::image::base::Image;
use servo_arc::Arc as ServoArc;
use std::sync::Arc;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;

#[derive(Debug)]
pub(crate) struct ReplacedContent {
    pub kind: ReplacedContentKind,
    pub intrinsic_size: physical::Vec2<Length>,
}

#[derive(Debug)]
pub(crate) enum ReplacedContentKind {
    Image(Option<Arc<Image>>),
}

impl ReplacedContent {
    pub fn for_element<'dom>(element: impl NodeExt<'dom>) -> Option<Self> {
        if let Some((image, intrinsic_size)) = element.as_image() {
            return Some(Self {
                kind: ReplacedContentKind::Image(image),
                intrinsic_size,
            });
        }
        None
    }

    pub fn make_fragments<'a>(
        &'a self,
        style: &ServoArc<ComputedValues>,
        size: flow_relative::Vec2<Length>,
    ) -> Vec<Fragment> {
        match &self.kind {
            ReplacedContentKind::Image(image) => image
                .as_ref()
                .and_then(|image| image.id)
                .map(|image_key| {
                    Fragment::Image(ImageFragment {
                        style: style.clone(),
                        rect: flow_relative::Rect {
                            start_corner: flow_relative::Vec2::zero(),
                            size,
                        },
                        image_key,
                    })
                })
                .into_iter()
                .collect(),
        }
    }

    // https://drafts.csswg.org/css2/visudet.html#inline-replaced-width
    // https://drafts.csswg.org/css2/visudet.html#inline-replaced-height
    pub fn used_size(
        &self,
        containing_block: &ContainingBlock,
        style: &ComputedValues,
    ) -> flow_relative::Vec2<Length> {
        let mode = style.writing_mode;
        // FIXME(nox): We shouldn't pretend we always have a fully known intrinsic size.
        let intrinsic_size = self.intrinsic_size.size_to_flow_relative(mode);
        // FIXME(nox): This can divide by zero.
        let intrinsic_ratio = intrinsic_size.inline.px() / intrinsic_size.block.px();

        let box_size = style.box_size().percentages_relative_to(containing_block);
        let min_box_size = style
            .min_box_size()
            .percentages_relative_to(containing_block)
            .auto_is(Length::zero);
        let max_box_size = style
            .max_box_size()
            .percentages_relative_to(containing_block);

        let clamp = |inline_size: Length, block_size: Length| {
            (
                inline_size.clamp_between_extremums(min_box_size.inline, max_box_size.inline),
                block_size.clamp_between_extremums(min_box_size.block, max_box_size.block),
            )
        };
        // https://drafts.csswg.org/css2/visudet.html#min-max-widths
        // https://drafts.csswg.org/css2/visudet.html#min-max-heights
        let (inline_size, block_size) = match (box_size.inline, box_size.block) {
            (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::LengthPercentage(block)) => {
                clamp(inline, block)
            },
            (LengthOrAuto::LengthPercentage(inline), LengthOrAuto::Auto) => {
                clamp(inline, inline / intrinsic_ratio)
            },
            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(block)) => {
                clamp(block * intrinsic_ratio, block)
            },
            (LengthOrAuto::Auto, LengthOrAuto::Auto) => {
                enum Violation {
                    None,
                    Below(Length),
                    Above(Length),
                }
                let violation = |size, min_size, mut max_size: Option<Length>| {
                    if let Some(max) = max_size.as_mut() {
                        max.max_assign(min_size);
                    }
                    if size < min_size {
                        return Violation::Below(min_size);
                    }
                    match max_size {
                        Some(max_size) if size > max_size => Violation::Above(max_size),
                        _ => Violation::None,
                    }
                };
                match (
                    violation(
                        intrinsic_size.inline,
                        min_box_size.inline,
                        max_box_size.inline,
                    ),
                    violation(intrinsic_size.block, min_box_size.block, max_box_size.block),
                ) {
                    // Row 1.
                    (Violation::None, Violation::None) => {
                        (intrinsic_size.inline, intrinsic_size.block)
                    },
                    // Row 2.
                    (Violation::Above(max_inline_size), Violation::None) => {
                        let block_size =
                            (max_inline_size / intrinsic_ratio).max(min_box_size.block);
                        (max_inline_size, block_size)
                    },
                    // Row 3.
                    (Violation::Below(min_inline_size), Violation::None) => {
                        let block_size =
                            (min_inline_size / intrinsic_ratio).clamp_below_max(max_box_size.block);
                        (min_inline_size, block_size)
                    },
                    // Row 4.
                    (Violation::None, Violation::Above(max_block_size)) => {
                        let inline_size =
                            (max_block_size * intrinsic_ratio).max(min_box_size.inline);
                        (inline_size, max_block_size)
                    },
                    // Row 5.
                    (Violation::None, Violation::Below(min_block_size)) => {
                        let inline_size =
                            (min_block_size * intrinsic_ratio).clamp_below_max(max_box_size.inline);
                        (inline_size, min_block_size)
                    },
                    // Rows 6-7.
                    (Violation::Above(max_inline_size), Violation::Above(max_block_size)) => {
                        if max_inline_size.px() / intrinsic_size.inline.px() <=
                            max_block_size.px() / intrinsic_size.block.px()
                        {
                            // Row 6.
                            let block_size =
                                (max_inline_size / intrinsic_ratio).max(min_box_size.block);
                            (max_inline_size, block_size)
                        } else {
                            // Row 7.
                            let inline_size =
                                (max_block_size * intrinsic_ratio).max(min_box_size.inline);
                            (inline_size, max_block_size)
                        }
                    },
                    // Rows 8-9.
                    (Violation::Below(min_inline_size), Violation::Below(min_block_size)) => {
                        if min_inline_size.px() / intrinsic_size.inline.px() <=
                            min_block_size.px() / intrinsic_size.block.px()
                        {
                            // Row 8.
                            let inline_size = (min_block_size * intrinsic_ratio)
                                .clamp_below_max(max_box_size.inline);
                            (inline_size, min_block_size)
                        } else {
                            // Row 9.
                            let block_size = (min_inline_size / intrinsic_ratio)
                                .clamp_below_max(max_box_size.block);
                            (min_inline_size, block_size)
                        }
                    },
                    // Row 10.
                    (Violation::Below(min_inline_size), Violation::Above(max_block_size)) => {
                        (min_inline_size, max_block_size)
                    },
                    // Row 11.
                    (Violation::Above(max_inline_size), Violation::Below(min_block_size)) => {
                        (max_inline_size, min_block_size)
                    },
                }
            },
        };
        flow_relative::Vec2 {
            inline: inline_size,
            block: block_size,
        }
    }
}
