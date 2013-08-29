/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, str, null_string};
use dom::node::{ScriptView, Node, DoctypeNodeTypeId};

/// The `DOCTYPE` tag.
pub struct DocumentType<View> {
    parent: Node<View>,
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
            parent: Node::new(DoctypeNodeTypeId),
            name: name,
            public_id: public_id,
            system_id: system_id,
            force_quirks: force_quirks,
        }
    }

    pub fn Name(&self) -> DOMString {
        str(self.name.clone())
    }

    pub fn PublicId(&self) -> DOMString {
        match self.public_id {
            Some(ref s) => str(s.clone()),
            None => null_string
        }
    }

    pub fn SystemId(&self) -> DOMString {
        match self.system_id {
            Some(ref s) => str(s.clone()),
            None => null_string
        }
    }
}
