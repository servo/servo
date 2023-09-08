/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![feature(once_cell)]

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
pub mod traversal;

pub use flow::BoxTree;
pub use fragment_tree::FragmentTree;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};

use crate::geom::LogicalVec2;

pub struct ContainingBlock<'a> {
    inline_size: Length,
    block_size: LengthOrAuto,
    style: &'a ComputedValues,
}

struct DefiniteContainingBlock<'a> {
    size: LogicalVec2<Length>,
    style: &'a ComputedValues,
}

impl<'a> From<&'_ DefiniteContainingBlock<'a>> for ContainingBlock<'a> {
    fn from(definite: &DefiniteContainingBlock<'a>) -> Self {
        ContainingBlock {
            inline_size: definite.size.inline,
            block_size: LengthOrAuto::LengthPercentage(definite.size.block),
            style: definite.style,
        }
    }
}
