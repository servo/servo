/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use selectors::matching::QuirksMode;
use style::dom::{TDocument, TNode};
use style::shared_lock::{
    SharedRwLock as StyleSharedRwLock, SharedRwLockReadGuard as StyleSharedRwLockReadGuard,
};
use style::stylist::Stylist;

use crate::dom::bindings::root::LayoutDom;
use crate::dom::document::{Document, LayoutDocumentHelpers};
use crate::dom::node::{LayoutNodeHelpers, Node, NodeFlags};
use crate::layout_dom::{ServoLayoutElement, ServoLayoutNode, ServoShadowRoot};

// A wrapper around documents that ensures ayout can only ever access safe properties.
#[derive(Clone, Copy)]
pub struct ServoLayoutDocument<'dom> {
    /// The wrapped private DOM Document
    document: LayoutDom<'dom, Document>,
}

impl<'ld> ::style::dom::TDocument for ServoLayoutDocument<'ld> {
    type ConcreteNode = ServoLayoutNode<'ld>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_js(self.document.upcast())
    }

    fn quirks_mode(&self) -> QuirksMode {
        self.document.quirks_mode()
    }

    fn is_html_document(&self) -> bool {
        self.document.is_html_document_for_layout()
    }

    fn shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }
}

impl<'ld> ServoLayoutDocument<'ld> {
    pub fn root_element(&self) -> Option<ServoLayoutElement<'ld>> {
        self.as_node()
            .dom_children()
            .flat_map(|n| n.as_element())
            .next()
    }

    pub fn needs_paint_from_layout(&self) {
        self.document.needs_paint_from_layout()
    }

    pub fn will_paint(&self) {
        self.document.will_paint()
    }

    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }

    pub fn shadow_roots(&self) -> Vec<ServoShadowRoot> {
        unsafe {
            self.document
                .shadow_roots()
                .iter()
                .map(|sr| {
                    debug_assert!(sr.upcast::<Node>().get_flag(NodeFlags::IS_CONNECTED));
                    ServoShadowRoot::from_layout_js(*sr)
                })
                .collect()
        }
    }

    pub fn flush_shadow_roots_stylesheets(
        &self,
        stylist: &mut Stylist,
        guard: &StyleSharedRwLockReadGuard,
    ) {
        unsafe {
            if !self.document.shadow_roots_styles_changed() {
                return;
            }
            self.document.flush_shadow_roots_stylesheets();
            for shadow_root in self.shadow_roots() {
                shadow_root.flush_stylesheets(stylist, guard);
            }
        }
    }

    pub fn from_layout_js(document: LayoutDom<'ld, Document>) -> Self {
        ServoLayoutDocument { document }
    }
}
