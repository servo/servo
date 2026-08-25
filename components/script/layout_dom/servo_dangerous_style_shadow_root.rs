/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

use std::fmt;

use style::dom::TShadowRoot;
use style::stylist::CascadeData;

use crate::dom::bindings::root::LayoutDom;
use crate::dom::shadowroot::ShadowRoot;
use crate::layout_dom::{ServoDangerousStyleElement, ServoDangerousStyleNode};

/// A wrapper around [`LayoutDom<_, ShadowRoot>`] to be used with `stylo` and `selectors`.
///
/// Note: This should only be used for `stylo` or `selectors interaction.
#[derive(Clone, Copy, PartialEq)]
pub struct ServoDangerousStyleShadowRoot<'dom> {
    /// The wrapped private DOM ShadowRoot.
    shadow_root: LayoutDom<'dom, ShadowRoot>,
}

impl fmt::Debug for ServoDangerousStyleShadowRoot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_node().fmt(f)
    }
}

impl<'dom> From<LayoutDom<'dom, ShadowRoot>> for ServoDangerousStyleShadowRoot<'dom> {
    fn from(shadow_root: LayoutDom<'dom, ShadowRoot>) -> Self {
        Self { shadow_root }
    }
}

impl<'dom> TShadowRoot for ServoDangerousStyleShadowRoot<'dom> {
    type ConcreteNode = ServoDangerousStyleNode<'dom>;

    fn as_node(&self) -> ServoDangerousStyleNode<'dom> {
        self.shadow_root.upcast().into()
    }

    fn host(&self) -> ServoDangerousStyleElement<'dom> {
        ServoDangerousStyleElement {
            element: self.shadow_root.get_host_for_layout(),
        }
    }

    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a,
    {
        Some(self.shadow_root.get_style_data_for_layout())
    }
}
