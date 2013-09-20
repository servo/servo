/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::DOMParserBinding;
use dom::bindings::codegen::DOMParserBinding::SupportedTypeValues::{Text_html, Text_xml};
use dom::bindings::utils::{DOMString, Fallible, WrapperCache, CacheableWrapper};
use dom::document::{AbstractDocument, Document, XML};
use dom::element::HTMLHtmlElementTypeId;
use dom::htmldocument::HTMLDocument;
use dom::htmlelement::HTMLElement;
use dom::htmlhtmlelement::HTMLHtmlElement;
use dom::node::Node;
use dom::window::Window;

pub struct DOMParser {
    owner: @mut Window, //XXXjdm Document instead?
    wrapper: WrapperCache
}

impl DOMParser {
    pub fn new(owner: @mut Window) -> @mut DOMParser {
        let parser = @mut DOMParser {
            owner: owner,
            wrapper: WrapperCache::new()
        };

        // TODO(tkuehn): This just handles the top-level page. Need to handle subframes.
        let cx = owner.page.js_info.get_ref().js_compartment.cx.ptr;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        parser.wrap_object_shared(cx, scope);
        parser
    }

    pub fn Constructor(owner: @mut Window) -> Fallible<@mut DOMParser> {
        Ok(DOMParser::new(owner))
    }

    pub fn ParseFromString(&self,
                           _s: &DOMString,
                           ty: DOMParserBinding::SupportedType)
                           -> Fallible<AbstractDocument> {
        unsafe {
            let root = @HTMLHtmlElement {
                htmlelement: HTMLElement::new(HTMLHtmlElementTypeId, ~"html")
            };

            let root = Node::as_abstract_node((*self.owner.page).js_info.get_ref().js_compartment.cx.ptr, root);
            let cx = (*self.owner.page).js_info.get_ref().js_compartment.cx.ptr;

            match ty {
                Text_html => {
                    Ok(HTMLDocument::new(root, None))
                }
                Text_xml => {
                    Ok(AbstractDocument::as_abstract(cx, @mut Document::new(root, None, XML)))
                }
                _ => {
                    fail!("unsupported document type")
                }
            }
        }
    }
}

