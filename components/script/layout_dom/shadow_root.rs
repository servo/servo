/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::marker::PhantomData;

use script_layout_interface::wrapper_traits::LayoutDataTrait;
use style::dom::TShadowRoot;
use style::shared_lock::SharedRwLockReadGuard as StyleSharedRwLockReadGuard;
use style::stylist::{CascadeData, Stylist};

use crate::dom::bindings::root::LayoutDom;
use crate::dom::shadowroot::{LayoutShadowRootHelpers, ShadowRoot};
use crate::layout_dom::{ServoLayoutElement, ServoLayoutNode};

pub struct ServoShadowRoot<'dom, LayoutDataType: LayoutDataTrait> {
    /// The wrapped private DOM ShadowRoot.
    shadow_root: LayoutDom<'dom, ShadowRoot>,

    /// A PhantomData that is used to track the type of the stored layout data.
    phantom: PhantomData<LayoutDataType>,
}

impl<'dom, LayoutDataType: LayoutDataTrait> Clone for ServoShadowRoot<'dom, LayoutDataType> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'dom, LayoutDataType: LayoutDataTrait> Copy for ServoShadowRoot<'dom, LayoutDataType> {}

impl<'a, LayoutDataType: LayoutDataTrait> PartialEq for ServoShadowRoot<'a, LayoutDataType> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.shadow_root == other.shadow_root
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> fmt::Debug for ServoShadowRoot<'dom, LayoutDataType> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_node().fmt(f)
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> ::style::dom::TShadowRoot
    for ServoShadowRoot<'dom, LayoutDataType>
{
    type ConcreteNode = ServoLayoutNode<'dom, LayoutDataType>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_js(self.shadow_root.upcast())
    }

    fn host(&self) -> ServoLayoutElement<'dom, LayoutDataType> {
        ServoLayoutElement::from_layout_js(self.shadow_root.get_host_for_layout())
    }

    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a,
    {
        Some(&self.shadow_root.get_style_data_for_layout())
    }
}

impl<'dom, LayoutDataType: LayoutDataTrait> ServoShadowRoot<'dom, LayoutDataType> {
    pub(super) fn from_layout_js(shadow_root: LayoutDom<'dom, ShadowRoot>) -> Self {
        ServoShadowRoot {
            shadow_root,
            phantom: PhantomData,
        }
    }

    pub unsafe fn flush_stylesheets(
        &self,
        stylist: &mut Stylist,
        guard: &StyleSharedRwLockReadGuard,
    ) {
        self.shadow_root
            .flush_stylesheets::<ServoLayoutElement<LayoutDataType>>(stylist, guard)
    }
}
