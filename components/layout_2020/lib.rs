/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

mod cell;
pub mod context;
pub mod display_list;
pub mod dom;
mod dom_traversal;
mod flexbox;
pub mod flow;
mod formatting_contexts;
mod fragment_tree;
pub mod geom;
mod layout_box_base;
mod taffy;
#[macro_use]
pub mod layout_debug;
mod construct_modern;
mod lists;
mod positioned;
pub mod query;
mod replaced;
mod sizing;
mod style_ext;
pub mod table;
pub mod traversal;

use app_units::Au;
pub use flow::BoxTree;
pub use fragment_tree::FragmentTree;
use geom::AuOrAuto;
use serde::Serialize;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;

use crate::geom::{LogicalVec2, SizeConstraint};
use crate::style_ext::AspectRatio;

/// Represents the set of constraints that we use when computing the min-content
/// and max-content inline sizes of an element.
pub(crate) struct ConstraintSpace {
    pub block_size: SizeConstraint,
    pub writing_mode: WritingMode,
    pub preferred_aspect_ratio: Option<AspectRatio>,
}

impl ConstraintSpace {
    fn new(
        block_size: SizeConstraint,
        writing_mode: WritingMode,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self {
            block_size,
            writing_mode,
            preferred_aspect_ratio,
        }
    }

    fn new_for_style_and_ratio(
        style: &ComputedValues,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self::new(
            SizeConstraint::default(),
            style.writing_mode,
            preferred_aspect_ratio,
        )
    }
}

/// A variant of [`ContainingBlock`] that allows an indefinite inline size.
/// Useful for code that is shared for both layout (where we know the inline size
/// of the containing block) and intrinsic sizing (where we don't know it).
pub(crate) struct IndefiniteContainingBlock {
    pub size: LogicalVec2<AuOrAuto>,
    pub writing_mode: WritingMode,
}

impl From<&ConstraintSpace> for IndefiniteContainingBlock {
    fn from(constraint_space: &ConstraintSpace) -> Self {
        Self {
            size: LogicalVec2 {
                inline: AuOrAuto::Auto,
                block: constraint_space.block_size.to_auto_or(),
            },
            writing_mode: constraint_space.writing_mode,
        }
    }
}

impl<'a> From<&'_ ContainingBlock<'a>> for IndefiniteContainingBlock {
    fn from(containing_block: &ContainingBlock<'a>) -> Self {
        Self {
            size: LogicalVec2 {
                inline: AuOrAuto::LengthPercentage(containing_block.size.inline),
                block: containing_block.size.block,
            },
            writing_mode: containing_block.style.writing_mode,
        }
    }
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for IndefiniteContainingBlock {
    fn from(containing_block: &DefiniteContainingBlock<'a>) -> Self {
        Self {
            size: containing_block
                .size
                .map(|v| AuOrAuto::LengthPercentage(*v)),
            writing_mode: containing_block.style.writing_mode,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ContainingBlockSize {
    inline: Au,
    block: AuOrAuto,
}

pub(crate) struct ContainingBlock<'a> {
    size: ContainingBlockSize,
    style: &'a ComputedValues,
}

struct DefiniteContainingBlock<'a> {
    size: LogicalVec2<Au>,
    style: &'a ComputedValues,
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for ContainingBlock<'a> {
    fn from(definite: &DefiniteContainingBlock<'a>) -> Self {
        ContainingBlock {
            size: ContainingBlockSize {
                inline: definite.size.inline,
                block: AuOrAuto::LengthPercentage(definite.size.block),
            },
            style: definite.style,
        }
    }
}
