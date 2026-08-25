/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

use crate::{DangerousStyleElement, DangerousStyleNode, LayoutElement, LayoutNode};

/// A trait that holds all the concrete implementations of the Layout DOM traits. This is
/// useful because it means that other types (specifically the implementation of the `Layout`
/// trait) can be parameterized over a single type (`LayoutDomTypeBundle`) rather than all of
/// the various Layout DOM trait implementations.
pub trait LayoutDomTypeBundle<'dom> {
    /// The concrete implementation of [`LayoutNode`] from `script`.
    type ConcreteLayoutNode: LayoutNode<'dom>;
    /// The concrete implementation of [`LayoutElement`] from `script`.
    type ConcreteLayoutElement: LayoutElement<'dom>;
    /// The concrete implementation of [`DangerousStyleNode`] from `script`.
    type ConcreteDangerousStyleNode: DangerousStyleNode<'dom>;
    /// The concrete implementation of [`DangerousStyleElement`] from `script`.
    type ConcreteDangerousStyleElement: DangerousStyleElement<'dom>;
}

// The type aliases below simplify extracting the concrete types out of the type bundle. It will be
// possible to simplify this once default associated types have landed and are stable:
// https://github.com/rust-lang/rust/issues/29661.

/// Type alias to extract `ConcreteLayoutNode` from a `LayoutDomTypeBundle` implementation.
pub type LayoutNodeOf<'dom, T> = <T as LayoutDomTypeBundle<'dom>>::ConcreteLayoutNode;

/// Type alias to extract `ConcreteLayoutElement` from a `LayoutDomTypeBundle` implementation.
pub type LayoutElementOf<'dom, T> = <T as LayoutDomTypeBundle<'dom>>::ConcreteLayoutElement;

/// Type alias to extract `ConcreteDangerousStyleNode` from a `LayoutDomTypeBundle` implementation.
pub type DangerousStyleNodeOf<'dom, T> =
    <T as LayoutDomTypeBundle<'dom>>::ConcreteDangerousStyleNode;

/// Type alias to extract `ConcreteDangerousStyleElement` from a `LayoutDomTypeBundle` implementation.
pub type DangerousStyleElementOf<'dom, T> =
    <T as LayoutDomTypeBundle<'dom>>::ConcreteDangerousStyleElement;
