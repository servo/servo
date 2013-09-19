/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::DOMString;
use dom::node::{ScriptView, Node, DoctypeNodeTypeId};

/// The `DOCTYPE` tag.
pub struct DocumentType<View> {
    node: Node<View>,
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

impl DocumentType<ScriptView> {
    /// Creates a new `DOCTYPE` tag.
    pub fn new(name: ~str,
               public_id: Option<~str>,
               system_id: Option<~str>,
               force_quirks: bool)
            -> DocumentType<ScriptView> {
        DocumentType {
            node: Node::new(DoctypeNodeTypeId),
            name: name,
            public_id: public_id,
            system_id: system_id,
            force_quirks: force_quirks,
        }
    }

    pub fn Name(&self) -> DOMString {
        Some(self.name.clone())
    }

    pub fn PublicId(&self) -> DOMString {
        self.public_id.clone()
    }

    pub fn SystemId(&self) -> DOMString {
        self.system_id.clone()
    }
}
