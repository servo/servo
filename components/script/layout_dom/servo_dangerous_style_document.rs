/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(missing_docs)]

use layout_api::DangerousStyleElement;
use selectors::matching::QuirksMode;
use style::dom::{TDocument, TNode};
use style::shared_lock::{
    SharedRwLock as StyleSharedRwLock, SharedRwLockReadGuard as StyleSharedRwLockReadGuard,
};
use style::stylist::Stylist;
use style::values::AtomIdent;

use crate::dom::bindings::root::LayoutDom;
use crate::dom::document::Document;
use crate::layout_dom::{ServoDangerousStyleElement, ServoDangerousStyleNode, ServoLayoutElement};

/// A wrapper around documents that ensures layout can only ever access safe properties.
///
/// TODO: This should become a trait like `LayoutNode`.
#[derive(Clone, Copy)]
pub struct ServoDangerousStyleDocument<'dom> {
    /// The wrapped private DOM Document
    document: LayoutDom<'dom, Document>,
}

impl<'dom> From<LayoutDom<'dom, Document>> for ServoDangerousStyleDocument<'dom> {
    fn from(document: LayoutDom<'dom, Document>) -> Self {
        Self { document }
    }
}

impl<'dom> ::style::dom::TDocument for ServoDangerousStyleDocument<'dom> {
    type ConcreteNode = ServoDangerousStyleNode<'dom>;

    fn as_node(&self) -> ServoDangerousStyleNode<'dom> {
        self.document.upcast().into()
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

    fn elements_with_id<'a>(
        &self,
        id: &AtomIdent,
    ) -> Result<&'a [ServoDangerousStyleElement<'dom>], ()>
    where
        Self: 'a,
    {
        let elements_with_id = self.document.elements_with_id(&id.0);

        // SAFETY: ServoDangerousStyleElement is known to have the same layout and alignment as
        // LayoutDom<Element>.
        #[expect(unsafe_code)]
        let result = unsafe {
            std::slice::from_raw_parts(
                elements_with_id.as_ptr() as *const ServoDangerousStyleElement<'dom>,
                elements_with_id.len(),
            )
        };
        Ok(result)
    }
}

impl<'dom> ServoDangerousStyleDocument<'dom> {
    /// Get the root node for this [`ServoDangerousStyleDocument`].
    pub fn root_element(&self) -> Option<ServoLayoutElement<'dom>> {
        self.as_node()
            .dom_children()
            .flat_map(|n| n.as_element())
            .next()
            .map(|element| element.layout_element())
    }

    /// Get the shared style lock for styling with Stylo for this [`ServoDangerousStyleDocument`].
    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }

    /// Flush the the stylesheets of all descendant shadow roots.
    pub fn flush_shadow_root_stylesheets_if_necessary(
        &self,
        stylist: &mut Stylist,
        guard: &StyleSharedRwLockReadGuard,
    ) {
        self.document
            .flush_shadow_root_stylesheets_if_necessary(stylist, guard);
    }
}
