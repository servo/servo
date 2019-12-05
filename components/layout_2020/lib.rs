/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![feature(exact_size_is_empty)]
#![feature(matches_macro)]

pub mod context;
pub mod data;
pub mod display_list;
mod dom_traversal;
mod element_data;
mod flow;
mod formatting_contexts;
mod fragments;
mod geom;
mod opaque_node;
mod positioned;
pub mod query;
mod replaced;
mod sizing;
mod style_ext;
pub mod traversal;
pub mod wrapper;

pub use flow::{BoxTreeRoot, FragmentTreeRoot};

use crate::geom::flow_relative::Vec2;
use crate::style_ext::ComputedValuesExt;
use style::computed_values::position::T as Position;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};
use style::Zero;

struct ContainingBlock {
    inline_size: Length,
    block_size: LengthOrAuto,
    mode: WritingMode,
}

struct DefiniteContainingBlock {
    size: Vec2<Length>,
    mode: WritingMode,
}

/// https://drafts.csswg.org/css2/visuren.html#relative-positioning
fn relative_adjustement(
    style: &ComputedValues,
    inline_size: Length,
    block_size: LengthOrAuto,
) -> Vec2<Length> {
    if style.get_box().position != Position::Relative {
        return Vec2::zero();
    }
    fn adjust(start: LengthOrAuto, end: LengthOrAuto) -> Length {
        match (start, end) {
            (LengthOrAuto::Auto, LengthOrAuto::Auto) => Length::zero(),
            (LengthOrAuto::Auto, LengthOrAuto::LengthPercentage(end)) => -end,
            (LengthOrAuto::LengthPercentage(start), _) => start,
        }
    }
    let block_size = block_size.auto_is(Length::zero);
    let box_offsets = style.box_offsets().map_inline_and_block_axes(
        |v| v.percentage_relative_to(inline_size),
        |v| v.percentage_relative_to(block_size),
    );
    Vec2 {
        inline: adjust(box_offsets.inline_start, box_offsets.inline_end),
        block: adjust(box_offsets.block_start, box_offsets.block_end),
    }
}
