/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! These types need to have the same size and alignment as the respectively corresponding
//! types in components/style/gecko/selector_parser.rs

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum PseudoClass {
    Bare,
    String(Box<[u16]>),
    Dir(Box<()>),
    MozAny(Box<[()]>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PseudoElement {
    A,
    B,
    Tree(Box<[String]>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Atom(usize);

#[derive(Clone, Eq, PartialEq)]
pub struct Impl;
