/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::reflector::Reflector;
use dom::bindings::js::JS;
use dom::document::Document;

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
    /// Input chunks received but not yet passed to the parser.
    pending_input: DOMRefCell<Vec<String>>,
}

impl ServoParser {
    pub fn new_inherited(document: &Document) -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
            pending_input: DOMRefCell::new(vec![]),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn has_pending_input(&self) -> bool {
        !self.pending_input.borrow().is_empty()
    }

    pub fn push_input_chunk(&self, chunk: String) {
        self.pending_input.borrow_mut().push(chunk);
    }

    pub fn take_next_input_chunk(&self) -> Option<String> {
        let mut pending_input = self.pending_input.borrow_mut();
        if pending_input.is_empty() {
            None
        } else {
            Some(pending_input.remove(0))
        }
    }
}
