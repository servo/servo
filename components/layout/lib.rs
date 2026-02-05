/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

//! Layout. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

mod accessibility_tree;
mod cell;
mod context;
mod display_list;
mod dom;
mod dom_traversal;
mod flexbox;
pub mod flow;
mod formatting_contexts;
mod fragment_tree;
pub mod geom;
mod layout_box_base;
mod layout_impl;
mod taffy;
#[macro_use]
mod construct_modern;
mod lists;
mod positioned;
mod query;
mod quotes;
mod replaced;
mod sizing;
mod style_ext;
pub mod table;
mod traversal;

use app_units::Au;
pub use cell::ArcRefCell;
pub(crate) use flow::BoxTree;
pub(crate) use fragment_tree::FragmentTree;
pub use layout_impl::LayoutFactoryImpl;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use style::logical_geometry::WritingMode;
use style::properties::ComputedValues;

use crate::geom::LogicalVec2;
use crate::sizing::SizeConstraint;
use crate::style_ext::AspectRatio;

/// At times, a style is "owned" by more than one layout object. For example, text
/// fragments need a handle on their parent inline box's style. In order to make
/// incremental layout easier to implement, another layer of shared ownership is added via
/// [`SharedStyle`]. This allows updating the style in originating layout object and
/// having all "depdendent" objects update automatically.
///
///  Note that this is not a cost-free data structure, so should only be
/// used when necessary.
pub(crate) type SharedStyle = ArcRefCell<ServoArc<ComputedValues>>;

/// Represents the set of constraints that we use when computing the min-content
/// and max-content inline sizes of an element.
pub(crate) struct ConstraintSpace<'a> {
    pub block_size: SizeConstraint,
    pub style: &'a ComputedValues,
    pub preferred_aspect_ratio: Option<AspectRatio>,
}

impl<'a> ConstraintSpace<'a> {
    fn new(
        block_size: SizeConstraint,
        style: &'a ComputedValues,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self {
            block_size,
            style,
            preferred_aspect_ratio,
        }
    }

    fn new_for_style_and_ratio(
        style: &'a ComputedValues,
        preferred_aspect_ratio: Option<AspectRatio>,
    ) -> Self {
        Self::new(SizeConstraint::default(), style, preferred_aspect_ratio)
    }
}

/// A variant of [`ContainingBlock`] that allows an indefinite inline size.
/// Useful for code that is shared for both layout (where we know the inline size
/// of the containing block) and intrinsic sizing (where we don't know it).
pub(crate) struct IndefiniteContainingBlock<'a> {
    pub size: LogicalVec2<Option<Au>>,
    pub style: &'a ComputedValues,
}

impl<'a> From<&ConstraintSpace<'a>> for IndefiniteContainingBlock<'a> {
    fn from(constraint_space: &ConstraintSpace<'a>) -> Self {
        Self {
            size: LogicalVec2 {
                inline: None,
                block: constraint_space.block_size.to_definite(),
            },
            style: constraint_space.style,
        }
    }
}

impl<'a> From<&'_ ContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    fn from(containing_block: &ContainingBlock<'a>) -> Self {
        Self {
            size: LogicalVec2 {
                inline: Some(containing_block.size.inline),
                block: containing_block.size.block.to_definite(),
            },
            style: containing_block.style,
        }
    }
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for IndefiniteContainingBlock<'a> {
    fn from(containing_block: &DefiniteContainingBlock<'a>) -> Self {
        Self {
            size: containing_block.size.map(|v| Some(*v)),
            style: containing_block.style,
        }
    }
}

#[derive(Clone, Debug, MallocSizeOf)]
pub(crate) struct ContainingBlockSize {
    inline: Au,
    block: SizeConstraint,
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
                block: SizeConstraint::Definite(definite.size.block),
            },
            style: definite.style,
        }
    }
}

/// Data that is propagated from ancestors to descendants during [`crate::flow::BoxTree`]
/// construction.  This allows data to flow in the reverse direction of the typical layout
/// propoagation, but only during `BoxTree` construction.
#[derive(Clone, Copy, Debug)]
struct PropagatedBoxTreeData {
    allow_percentage_column_in_tables: bool,
}

impl Default for PropagatedBoxTreeData {
    fn default() -> Self {
        Self {
            allow_percentage_column_in_tables: true,
        }
    }
}

impl PropagatedBoxTreeData {
    fn disallowing_percentage_table_columns(&self) -> PropagatedBoxTreeData {
        Self {
            allow_percentage_column_in_tables: false,
        }
    }
}
