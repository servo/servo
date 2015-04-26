/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use legacy::{IntegerAttribute, LengthAttribute, UnsignedIntegerAttribute};
use properties::PropertyDeclaration;
use util::str::LengthOrPercentageOrAuto;

use selectors::matching::DeclarationBlock;
use selectors::smallvec::VecLike;
pub use selectors::tree::{TNode, TElement};

pub trait TElementAttributes : Copy {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    fn get_length_attribute(self, attribute: LengthAttribute) -> LengthOrPercentageOrAuto;
    fn get_integer_attribute(self, attribute: IntegerAttribute) -> Option<i32>;
    fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute) -> Option<u32>;
}
