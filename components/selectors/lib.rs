/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
#[macro_use] extern crate matches;
extern crate fnv;
extern crate precomputed_hash;
extern crate smallvec;

pub mod arcslice;
pub mod bloom;
pub mod matching;
pub mod parser;
mod tree;
pub mod visitor;

pub use parser::{SelectorImpl, Parser, SelectorList};
pub use tree::Element;
pub use tree::{MatchAttr, MatchAttrGeneric};
