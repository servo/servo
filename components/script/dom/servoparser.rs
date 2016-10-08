/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::reflector::Reflector;
use dom::bindings::js::JS;
use dom::document::Document;

#[dom_struct]
pub struct ServoParser {
    reflector: Reflector,
    /// The document associated with this parser.
    document: JS<Document>,
}

impl ServoParser {
    pub fn new_inherited(document: &Document) -> Self {
        ServoParser {
            reflector: Reflector::new(),
            document: JS::from_ref(document),
        }
    }

    pub fn document(&self) -> &Document {
        &self.document
    }
}
