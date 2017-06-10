/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cssparser::ToCss;
use gecko_like_types;
use gecko_like_types::*;
use parser;
use parser::*;
use precomputed_hash::PrecomputedHash;
use std::fmt;
use visitor::SelectorVisitor;

size_of_test!(size_of_selector, Selector<Impl>, 8);
size_of_test!(size_of_pseudo_element, gecko_like_types::PseudoElement, 1);

size_of_test!(size_of_component, Component<Impl>, 32);
size_of_test!(size_of_pseudo_class, PseudoClass, 24);

impl parser::PseudoElement for gecko_like_types::PseudoElement {
    type Impl = Impl;
}

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
    type PseudoElement = gecko_like_types::PseudoElement;

    #[inline]
    fn is_active_or_hover(_pseudo_class: &Self::NonTSPseudoClass) -> bool {
        unimplemented!()
    }
}

impl SelectorMethods for PseudoClass {
    type Impl = Impl;

    fn visit<V>(&self, _visitor: &mut V) -> bool
        where V: SelectorVisitor<Impl = Self::Impl> { unimplemented!() }
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write { unimplemented!() }
}

impl ToCss for gecko_like_types::PseudoElement {
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
