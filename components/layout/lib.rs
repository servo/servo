/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

//! Layout. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

mod accessibility;
mod cell;
mod containing_block;
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
mod layout_root;
mod lists;
mod positioned;
mod query;
mod quotes;
mod replaced;
mod selection;
mod sizing;
mod style_ext;
pub mod table;
mod traversal;

pub use cell::ArcRefCell;
pub(crate) use flow::BoxTree;
pub(crate) use fragment_tree::FragmentTree;
pub use layout_impl::LayoutFactoryImpl;
use malloc_size_of_derive::MallocSizeOf;
use servo_arc::Arc as ServoArc;
use style::properties::ComputedValues;

/// At times, a style is "owned" by more than one layout object. For example, text
/// fragments need a handle on their parent inline box's style. In order to make
/// incremental layout easier to implement, another layer of shared ownership is added via
/// [`SharedStyle`]. This allows updating the style in originating layout object and
/// having all "depdendent" objects update automatically.
///
///  Note that this is not a cost-free data structure, so should only be
/// used when necessary.
pub(crate) type SharedStyle = ArcRefCell<ServoArc<ComputedValues>>;

/// Data that is propagated from ancestors to descendants during [`crate::flow::BoxTree`]
/// construction.  This allows data to flow in the reverse direction of the typical layout
/// propoagation, but only during `BoxTree` construction.
#[derive(Clone, Copy, Debug, MallocSizeOf)]
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
