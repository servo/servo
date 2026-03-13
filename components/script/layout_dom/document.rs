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

use style::values::AtomIdent;
impl<'ld> ::style::dom::TDocument for ServoLayoutDocument<'ld> {
    type ConcreteNode = ServoLayoutNode<'ld>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_dom(self.document.upcast())
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

    fn elements_with_id<'a>(&self, id: &AtomIdent) -> Result<&'a [ServoLayoutElement<'ld>], ()>
    where
        Self: 'a,
    {
        let elements_with_id = self.document.elements_with_id(&id.0);

        // SAFETY: ServoLayoutElement is known to have the same layout and alignment as LayoutDom<Element>
        let result = unsafe {
            std::slice::from_raw_parts(
                elements_with_id.as_ptr() as *const ServoLayoutElement<'ld>,
                elements_with_id.len(),
            )
        };
        Ok(result)
    }
}

impl<'ld> ServoLayoutDocument<'ld> {
    pub fn root_element(&self) -> Option<ServoLayoutElement<'ld>> {
        self.as_node()
            .dom_children()
            .flat_map(|n| n.as_element())
            .next()
    }

    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }

    pub fn shadow_roots(&self) -> Vec<ServoShadowRoot<'_>> {
        unsafe {
            self.document
                .shadow_roots()
                .iter()
                .map(|sr| {
                    debug_assert!(sr.upcast::<Node>().get_flag(NodeFlags::IS_CONNECTED));
                    ServoShadowRoot::from_layout_dom(*sr)
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

    pub(crate) fn from_layout_dom(document: LayoutDom<'ld, Document>) -> Self {
        ServoLayoutDocument { document }
    }
}
