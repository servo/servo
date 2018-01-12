/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Make |cargo bench| work.
#![cfg_attr(feature = "bench", feature(test))]

#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
#[macro_use] extern crate log;
#[macro_use] extern crate matches;
extern crate fnv;
extern crate phf;
extern crate precomputed_hash;
extern crate servo_arc;
extern crate smallvec;

pub mod attr;
pub mod bloom;
mod builder;
pub mod context;
pub mod matching;
mod nth_index_cache;
pub mod parser;
pub mod sink;
mod tree;
pub mod visitor;

pub use nth_index_cache::NthIndexCache;
pub use parser::{SelectorImpl, Parser, SelectorList};
pub use tree::{Element, OpaqueElement};
