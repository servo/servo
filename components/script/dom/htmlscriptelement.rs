/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::HTMLScriptElementDerived;
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast};
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{HTMLScriptElementTypeId, Element, AttributeHandlers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};

use servo_util::str::{DOMString, HTML_SPACE_CHARACTERS, StaticStringVec};

#[jstraceable]
#[must_root]
pub struct HTMLScriptElement {
    pub htmlelement: HTMLElement,
}

impl HTMLScriptElementDerived for EventTarget {
    fn is_htmlscriptelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLScriptElementTypeId))
    }
}

impl HTMLScriptElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLScriptElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLScriptElementBinding::Wrap)
    }
}

pub trait HTMLScriptElementHelpers {
    /// Prepare a script (<http://www.whatwg.org/html/#prepare-a-script>),
    /// steps 6 and 7.
    fn is_javascript(self) -> bool;
}

/// Supported script types as defined by
/// <http://whatwg.org/html/#support-the-scripting-language>.
static SCRIPT_JS_MIMES: StaticStringVec = &[
    "application/ecmascript",
    "application/javascript",
    "application/x-ecmascript",
    "application/x-javascript",
    "text/ecmascript",
    "text/javascript",
    "text/javascript1.0",
    "text/javascript1.1",
    "text/javascript1.2",
    "text/javascript1.3",
    "text/javascript1.4",
    "text/javascript1.5",
    "text/jscript",
    "text/livescript",
    "text/x-ecmascript",
    "text/x-javascript",
];

impl<'a> HTMLScriptElementHelpers for JSRef<'a, HTMLScriptElement> {
    fn is_javascript(self) -> bool {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        match element.get_attribute(ns!(""), "type").root().map(|s| s.Value()) {
            Some(ref s) if s.is_empty() => {
                // type attr exists, but empty means js
                debug!("script type empty, inferring js");
                true
            },
            Some(ref s) => {
                debug!("script type={:s}", *s);
                SCRIPT_JS_MIMES.contains(&s.as_slice().trim_chars(HTML_SPACE_CHARACTERS))
            },
            None => {
                debug!("no script type");
                match element.get_attribute(ns!(""), "language").root().map(|s| s.Value()) {
                    Some(ref s) if s.is_empty() => {
                        debug!("script language empty, inferring js");
                        true
                    },
                    Some(ref s) => {
                        debug!("script language={:s}", *s);
                        SCRIPT_JS_MIMES.contains(&format!("text/{}", s).as_slice())
                    },
                    None => {
                        debug!("no script type or language, inferring js");
                        true
                    }
                }
            }
        }
    }
}

impl<'a> HTMLScriptElementMethods for JSRef<'a, HTMLScriptElement> {
    fn Src(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        element.get_url_attribute("src")
    }

    // http://www.whatwg.org/html/#dom-script-text
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        Node::collect_text_contents(node.children())
    }

    // http://www.whatwg.org/html/#dom-script-text
    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl Reflectable for HTMLScriptElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
