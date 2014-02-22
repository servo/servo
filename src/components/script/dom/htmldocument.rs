/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::codegen::InheritTypes::HTMLDocumentDerived;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, HTML, HTMLDocumentTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::node::DocumentNodeTypeId;
use dom::window::Window;

use extra::url::Url;

#[deriving(Encodable)]
pub struct HTMLDocument {
    parent: Document
}

impl HTMLDocumentDerived for EventTarget {
    fn is_htmldocument(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(DocumentNodeTypeId(HTMLDocumentTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDocument {
    pub fn new_inherited(window: JS<Window>, url: Option<Url>) -> HTMLDocument {
        HTMLDocument {
            parent: Document::new_inherited(window, url, HTML, None)
        }
    }

    pub fn new(window: &JS<Window>, url: Option<Url>) -> JS<HTMLDocument> {
        let document = HTMLDocument::new_inherited(window.clone(), url);
        Document::reflect_document(~document, window, HTMLDocumentBinding::Wrap)
    }
}

impl Reflectable for HTMLDocument {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.parent.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.parent.mut_reflector()
    }
}
