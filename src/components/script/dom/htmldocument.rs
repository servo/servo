/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::utils::{Reflectable, Reflector, Traceable};
use dom::document::{AbstractDocument, Document, HTML};
use dom::htmlcollection::HTMLCollection;
use dom::window::Window;
use servo_util::namespace::Null;

use extra::url::Url;
use js::jsapi::JSTracer;

pub struct HTMLDocument {
    parent: Document
}

impl HTMLDocument {
    pub fn new_inherited(window: @mut Window, url: Option<Url>) -> HTMLDocument {
        HTMLDocument {
            parent: Document::new_inherited(window, url, HTML, None)
        }
    }

    pub fn new(window: @mut Window, url: Option<Url>) -> AbstractDocument {
        let document = HTMLDocument::new_inherited(window, url);
        Document::reflect_document(@mut document, window, HTMLDocumentBinding::Wrap)
    }
}

impl HTMLDocument {
    pub fn Images(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| "img" == elem.tag_name)
    }

    pub fn Embeds(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| "embed" == elem.tag_name)
    }

    pub fn Plugins(&self) -> @mut HTMLCollection {
        self.Embeds()
    }

    pub fn Links(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| {
            ("a" == elem.tag_name || "area" == elem.tag_name) &&
            elem.get_attribute(Null, "href").is_some()
        })
    }

    pub fn Forms(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| "form" == elem.tag_name)
    }

    pub fn Scripts(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| "script" == elem.tag_name)
    }

    pub fn Anchors(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| {
            "a" == elem.tag_name && elem.get_attribute(Null, "name").is_some()
        })
    }

    pub fn Applets(&self) -> @mut HTMLCollection {
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

impl Traceable for HTMLDocument {
    fn trace(&self, tracer: *mut JSTracer) {
        self.parent.trace(tracer);
    }
}
