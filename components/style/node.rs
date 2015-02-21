/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use cssparser::RGBA;
use legacy::{IntegerAttribute, LengthAttribute, SimpleColorAttribute, UnsignedIntegerAttribute};
use util::str::LengthOrPercentageOrAuto;

pub use selectors::tree::{TNode, TElement};

pub trait TElementAttributes : Copy {
    fn get_length_attribute(self, attribute: LengthAttribute) -> LengthOrPercentageOrAuto;
    fn get_integer_attribute(self, attribute: IntegerAttribute) -> Option<i32>;
    fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute) -> Option<u32>;
    fn get_simple_color_attribute(self, attribute: SimpleColorAttribute) -> Option<RGBA>;
}
