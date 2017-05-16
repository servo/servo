/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use gecko_like_types::*;
use parser::*;
use precomputed_hash::PrecomputedHash;
use std::fmt;
use visitor::SelectorVisitor;

size_of_test!(size_of_selector, Selector<Impl>, 72);
size_of_test!(size_of_pseudo_element, PseudoElementSelector, 16);
size_of_test!(size_of_selector_inner, SelectorInner<Impl>, 40);
size_of_test!(size_of_complex_selector, ComplexSelector<Impl>, 24);

size_of_test!(size_of_component, Component<Impl>, 64);
size_of_test!(size_of_attr_selector, AttrSelector<Impl>, 48);
size_of_test!(size_of_pseudo_class, PseudoClass, 24);


// Boilerplate

impl SelectorImpl for Impl {
    type AttrValue = Atom;
    type Identifier = Atom;
    type ClassName = Atom;
    type LocalName = Atom;
    type NamespaceUrl = Atom;
    type NamespacePrefix = Atom;
    type BorrowedLocalName = Atom;
    type BorrowedNamespaceUrl = Atom;
    type NonTSPseudoClass = PseudoClass;
    type PseudoElementSelector = PseudoElementSelector;
}

impl SelectorMethods for PseudoClass {
    type Impl = Impl;

    fn visit<V>(&self, _visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Self::Impl> { unimplemented!() }
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write { unimplemented!() }
}

impl ToCss for PseudoElementSelector {
    fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write { unimplemented!() }
}

impl fmt::Display for Atom {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result { unimplemented!() }
}

impl From<String> for Atom {
    fn from(_: String) -> Self { unimplemented!() }
}

impl<'a> From<&'a str> for Atom {
    fn from(_: &'a str) -> Self { unimplemented!() }
}

impl PrecomputedHash for Atom {
    fn precomputed_hash(&self) -> u32 { unimplemented!() }
}
