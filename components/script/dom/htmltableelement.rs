/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::Bindings::HTMLTableElementBinding::HTMLTableElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, NodeCast, HTMLTableCaptionElementCast};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableCaptionElementTypeId;
use dom::element::HTMLTableElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmltablecaptionelement::HTMLTableCaptionElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTableElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableElementDerived for EventTarget {
    fn is_htmltableelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableElementTypeId))
    }
}

impl HTMLTableElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

impl<'a> HTMLTableElementMethods for JSRef<'a, HTMLTableElement> {
    //  http://www.whatwg.org/html/#dom-table-caption
    fn GetCaption(self) -> Option<Temporary<HTMLTableCaptionElement>> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.children().find(|child| {
            child.type_id() == ElementNodeTypeId(HTMLTableCaptionElementTypeId)
        }).map(|node| {
            Temporary::from_rooted(HTMLTableCaptionElementCast::to_ref(node).unwrap())
        })
    }

    // http://www.whatwg.org/html/#dom-table-caption
    fn SetCaption(self, new_caption: Option<JSRef<HTMLTableCaptionElement>>) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let old_caption = self.GetCaption();

        match old_caption {
            Some(htmlelem) => {
                let htmlelem_root = htmlelem.root();
                let old_caption_node: JSRef<Node> = NodeCast::from_ref(*htmlelem_root);
                assert!(node.RemoveChild(old_caption_node).is_ok());
            }
            None => ()
        }

        new_caption.map(|caption| {
            let new_caption_node: JSRef<Node> = NodeCast::from_ref(caption);
            assert!(node.AppendChild(new_caption_node).is_ok());
        });
    }
}
