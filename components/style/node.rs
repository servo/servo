/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use legacy::UnsignedIntegerAttribute;
use properties::PropertyDeclaration;
use util::smallvec::VecLike;

use selectors::matching::DeclarationBlock;
pub use selectors::tree::{TNode, TElement};
use string_cache::{Atom, Namespace};

pub trait TElementAttributes<'a> : Copy {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute) -> Option<u32>;

    fn get_attr(self, namespace: &Namespace, attr: &Atom) -> Option<&'a str>;
    fn get_attrs(self, attr: &Atom) -> Vec<&'a str>;
}
