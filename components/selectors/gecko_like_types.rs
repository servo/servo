/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! These types need to have the same size and alignment as the respectively corresponding
//! types in components/style/gecko/selector_parser.rs

#[derive(Eq, PartialEq, Clone, Debug)]
#[allow(dead_code)]
pub enum PseudoClass {
    Bare,
    String(Box<[u16]>),
    MozAny(Box<[()]>),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum PseudoElement {
    A,
    B,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PseudoElementSelector(PseudoElement, u64);

#[derive(Eq, PartialEq, Clone, Debug, Default)]
pub struct Atom(usize);

#[derive(Eq, PartialEq, Clone)]
pub struct Impl;
