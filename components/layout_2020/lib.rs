/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

mod cell;
pub mod context;
pub mod data;
pub mod display_list;
mod dom_traversal;
pub mod element_data;
mod flexbox;
pub mod flow;
mod formatting_contexts;
mod fragments;
pub mod geom;
#[macro_use]
pub mod layout_debug;
mod lists;
mod opaque_node;
mod positioned;
pub mod query;
mod replaced;
mod sizing;
mod style_ext;
pub mod traversal;
pub mod wrapper;

pub use flow::{BoxTree, FragmentTree};

use crate::geom::flow_relative::Vec2;
use style::properties::ComputedValues;
use style::values::computed::{Length, LengthOrAuto};

pub struct ContainingBlock<'a> {
    inline_size: Length,
    block_size: LengthOrAuto,
    style: &'a ComputedValues,
}

struct DefiniteContainingBlock<'a> {
    size: Vec2<Length>,
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
