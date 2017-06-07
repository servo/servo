/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[macro_use] extern crate bitflags;
#[macro_use] extern crate cssparser;
#[macro_use] extern crate log;
#[macro_use] extern crate matches;
extern crate fnv;
extern crate phf;
extern crate precomputed_hash;
#[cfg(test)] #[macro_use] extern crate size_of_test;
extern crate servo_arc;
extern crate smallvec;

pub mod attr;
pub mod bloom;
pub mod matching;
pub mod parser;
#[cfg(test)] mod size_of_tests;
#[cfg(any(test, feature = "gecko_like_types"))] pub mod gecko_like_types;
mod tree;
pub mod visitor;

pub use parser::{SelectorImpl, Parser, SelectorList};
pub use tree::Element;
