/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::codegen::InheritTypes::HTMLDocumentDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Reflector, Traceable};
use dom::document::{Document, HTML, HTMLDocumentTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::namespace::Null;
use dom::node::DocumentNodeTypeId;
use dom::window::Window;

use js::jsapi::JSTracer;
use std::str::eq_slice;
use style::TElement;

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
    pub fn new_inherited(window: JSManaged<Window>) -> HTMLDocument {
        HTMLDocument {
            parent: Document::new_inherited(window, HTML)
        }
    }

    pub fn new(window: JSManaged<Window>) -> JSManaged<HTMLDocument> {
        let document = HTMLDocument::new_inherited(window);
        Document::reflect_document(~document, window, HTMLDocumentBinding::Wrap)
    }
}

impl HTMLDocument {
    pub fn Images(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "img"))
    }

    pub fn Embeds(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "embed"))
    }

    pub fn Plugins(&self) -> @mut HTMLCollection {
        self.Embeds()
    }

    pub fn Links(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem|
            (eq_slice(elem.tag_name, "a") || eq_slice(elem.tag_name, "area"))
            && elem.get_attr(Null, "href").is_some())
    }

    pub fn Forms(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "form"))
    }

    pub fn Scripts(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "script"))
    }

    pub fn Anchors(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem|
            eq_slice(elem.tag_name, "a") && elem.get_attr(Null, "name").is_some())
    }

    pub fn Applets(&self) -> @mut HTMLCollection {
        // FIXME: This should be return OBJECT elements containing applets.
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "applet"))
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

impl Traceable for HTMLDocument {
    fn trace(&self, tracer: *mut JSTracer) {
        self.parent.trace(tracer);
    }
}
