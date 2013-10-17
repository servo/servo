/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDocumentBinding;
use dom::bindings::utils::{Reflectable, Reflector, Traceable};
use dom::document::{AbstractDocument, Document, ReflectableDocument, HTML};
use dom::element::HTMLHeadElementTypeId;
use dom::htmlcollection::HTMLCollection;
use dom::node::{AbstractNode, ScriptView, ElementNodeTypeId};
use dom::window::Window;

use js::jsapi::{JSObject, JSContext, JSTracer};

use servo_util::tree::{TreeNodeRef, ElementLike};

use std::ptr;
use std::str::eq_slice;

pub struct HTMLDocument {
    parent: Document
}

impl HTMLDocument {
    pub fn new(window: @mut Window) -> AbstractDocument {
        let doc = @mut HTMLDocument {
            parent: Document::new(window, HTML)
        };

        AbstractDocument::as_abstract(window.get_cx(), doc)
    }
}

impl ReflectableDocument for HTMLDocument {
    fn init_reflector(@mut self, cx: *JSContext) {
        self.wrap_object_shared(cx, ptr::null()); //XXXjdm a proper scope would be nice
    }

    fn init_node(@mut self, doc: AbstractDocument) {
        self.parent.node.set_owner_doc(doc);
        self.parent.node.add_to_doc(AbstractNode::from_document(doc), doc);
    }
}

impl HTMLDocument {
    pub fn GetHead(&self) -> Option<AbstractNode<ScriptView>> {
        match self.parent.GetDocumentElement() {
            None => None,
            Some(root) => root.traverse_preorder().find(|child| {
                child.type_id() == ElementNodeTypeId(HTMLHeadElementTypeId)
            })
        }
    }

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
            && elem.get_attr("href").is_some())
    }

    pub fn Forms(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "form"))
    }

    pub fn Scripts(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem| eq_slice(elem.tag_name, "script"))
    }

    pub fn Anchors(&self) -> @mut HTMLCollection {
        self.parent.createHTMLCollection(|elem|
            eq_slice(elem.tag_name, "a") && elem.get_attr("name").is_some())
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

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        HTMLDocumentBinding::Wrap(cx, scope, self)
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.parent.GetParentObject(cx)
    }
}

impl Traceable for HTMLDocument {
    fn trace(&self, tracer: *mut JSTracer) {
        self.parent.trace(tracer);
    }
}
