/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traits that nodes must implement. Breaks the otherwise-cyclic dependency between layout and
//! style.

use properties::PropertyDeclaration;
use selectors::matching::DeclarationBlock;
use smallvec::VecLike;
use string_cache::{Atom, Namespace};

pub trait TElementAttributes {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;

    fn get_attr<'a>(&'a self, namespace: &Namespace, attr: &Atom) -> Option<&'a str>;
    fn get_attrs<'a>(&'a self, attr: &Atom) -> Vec<&'a str>;
}
