/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use selectors::gecko_like_types as dummies;
use std::mem::{size_of, align_of};
use style;
use style::gecko::selector_parser as real;

#[test]
fn size_of_selectors_dummy_types() {
    assert_eq!(size_of::<dummies::PseudoClass>(), size_of::<real::NonTSPseudoClass>());
    assert_eq!(align_of::<dummies::PseudoClass>(), align_of::<real::NonTSPseudoClass>());

    assert_eq!(size_of::<dummies::PseudoElementSelector>(), size_of::<real::PseudoElementSelector>());
    assert_eq!(align_of::<dummies::PseudoElementSelector>(), align_of::<real::PseudoElementSelector>());

    assert_eq!(size_of::<dummies::Atom>(), size_of::<style::Atom>());
    assert_eq!(align_of::<dummies::Atom>(), align_of::<style::Atom>());
}

#[test]
fn size_of_property_declaration() {
    ::style::properties::test_size_of_property_declaration();
}

#[test]
fn size_of_specified_values() {
    ::style::properties::test_size_of_specified_values();
}
