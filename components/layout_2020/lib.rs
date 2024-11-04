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
#[macro_use]
pub mod layout_debug;
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
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;

use crate::geom::LogicalVec2;

/// A containing block useful for calculating inline content sizes, which may
/// have inline sizes that depend on block sizes due to aspect ratio.
pub(crate) struct IndefiniteContainingBlock {
    pub size: LogicalVec2<AuOrAuto>,
    pub writing_mode: WritingMode,
}

impl IndefiniteContainingBlock {
    fn new_for_writing_mode(writing_mode: WritingMode) -> Self {
        Self::new_for_writing_mode_and_block_size(writing_mode, AuOrAuto::Auto)
    }

    /// Creates an [`IndefiniteContainingBlock`] with the provided style and block size,
    /// and the inline size is set to auto.
    /// This is useful when finding the min-content or max-content size of an element,
    /// since then we ignore its 'inline-size', 'min-inline-size' and 'max-inline-size'.
    fn new_for_writing_mode_and_block_size(
        writing_mode: WritingMode,
        block_size: AuOrAuto,
    ) -> Self {
        Self {
            size: LogicalVec2 {
                inline: AuOrAuto::Auto,
                block: block_size,
            },
            writing_mode,
        }
    }
}

impl<'a> From<&'_ ContainingBlock<'a>> for IndefiniteContainingBlock {
    fn from(containing_block: &ContainingBlock<'a>) -> Self {
        Self {
            size: LogicalVec2 {
                inline: AuOrAuto::LengthPercentage(containing_block.inline_size),
                block: containing_block.block_size,
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

pub struct ContainingBlock<'a> {
    inline_size: Au,
    block_size: AuOrAuto,
    style: &'a ComputedValues,
}

struct DefiniteContainingBlock<'a> {
    size: LogicalVec2<Au>,
    style: &'a ComputedValues,
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for ContainingBlock<'a> {
    fn from(definite: &DefiniteContainingBlock<'a>) -> Self {
        ContainingBlock {
            inline_size: definite.size.inline,
            block_size: AuOrAuto::LengthPercentage(definite.size.block),
            style: definite.style,
        }
    }
}
