/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::codegen::InheritTypes::HTMLDocumentDerived;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, HTML, HTMLDocumentTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::node::DocumentNodeTypeId;
use dom::window::Window;
use servo_util::namespace::Null;

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

impl HTMLDocument {
    pub fn Images(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| "img" == elem.tag_name)
    }

    pub fn Embeds(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| "embed" == elem.tag_name)
    }

    pub fn Plugins(&self) -> JS<HTMLCollection> {
        self.Embeds()
    }

    pub fn Links(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| {
            ("a" == elem.tag_name || "area" == elem.tag_name) &&
            elem.get_attribute(Null, "href").is_some()
        })
    }

    pub fn Forms(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| "form" == elem.tag_name)
    }

    pub fn Scripts(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| "script" == elem.tag_name)
    }

    pub fn Anchors(&self) -> JS<HTMLCollection> {
        self.parent.createHTMLCollection(|elem| {
            "a" == elem.tag_name && elem.get_attribute(Null, "name").is_some()
        })
    }

    pub fn Applets(&self) -> JS<HTMLCollection> {
        // FIXME: This should be return OBJECT elements containing applets.
        self.parent.createHTMLCollection(|elem| "applet" == elem.tag_name)
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
