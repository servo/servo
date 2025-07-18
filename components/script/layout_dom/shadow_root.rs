/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use style::dom::TShadowRoot;
use style::shared_lock::SharedRwLockReadGuard as StyleSharedRwLockReadGuard;
use style::stylist::{CascadeData, Stylist};

use crate::dom::bindings::root::LayoutDom;
use crate::dom::shadowroot::{LayoutShadowRootHelpers, ShadowRoot};
use crate::layout_dom::{ServoLayoutElement, ServoLayoutNode};

#[derive(Clone, Copy, PartialEq)]
pub struct ServoShadowRoot<'dom> {
    /// The wrapped private DOM ShadowRoot.
    shadow_root: LayoutDom<'dom, ShadowRoot>,
}

impl fmt::Debug for ServoShadowRoot<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_node().fmt(f)
    }
}

impl<'dom> TShadowRoot for ServoShadowRoot<'dom> {
    type ConcreteNode = ServoLayoutNode<'dom>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_js(self.shadow_root.upcast())
    }

    fn host(&self) -> ServoLayoutElement<'dom> {
        ServoLayoutElement::from_layout_js(self.shadow_root.get_host_for_layout())
    }

    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a,
    {
        Some(self.shadow_root.get_style_data_for_layout())
    }
}

impl<'dom> ServoShadowRoot<'dom> {
    pub(super) fn from_layout_js(shadow_root: LayoutDom<'dom, ShadowRoot>) -> Self {
        ServoShadowRoot { shadow_root }
    }

    /// Flush the stylesheets for the underlying shadow root.
    ///
    /// # Safety
    ///
    /// This modifies a DOM object, so should care should be taken that only one
    /// thread has a reference to this object.
    pub unsafe fn flush_stylesheets(
        &self,
        stylist: &mut Stylist,
        guard: &StyleSharedRwLockReadGuard,
    ) {
        unsafe {
            self.shadow_root
                .flush_stylesheets::<ServoLayoutElement>(stylist, guard)
        }
    }
}
