/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use content::content_task::global_content;
use dom::bindings::codegen::DOMParserBinding;
use dom::bindings::utils::{DOMString, ErrorResult, WrapperCache, CacheableWrapper};
use dom::document::Document;
use dom::element::{Element, HTMLHtmlElement, HTMLHtmlElementTypeId};
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

        let cx = global_content().compartment.get().cx.ptr;
        let cache = owner.get_wrappercache();
        let scope = cache.get_wrapper();
        parser.wrap_object_shared(cx, scope);
        parser
    }

    pub fn Constructor(owner: @mut Window, _rv: &mut ErrorResult) -> @mut DOMParser {
        DOMParser::new(owner)
    }

    pub fn ParseFromString(&self,
                           _s: DOMString,
                           _type: DOMParserBinding::SupportedType,
                           _rv: &mut ErrorResult)
                           -> @mut Document {
        unsafe {
            let root = ~HTMLHtmlElement {
                parent: Element::new(HTMLHtmlElementTypeId, ~"html")
            };

            let root = Node::as_abstract_node(root);
            Document(root, None)
        }
    }
}

