/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLImageElementBinding;
use dom::bindings::codegen::InheritTypes::{NodeCast, HTMLImageElementDerived};
use dom::bindings::codegen::InheritTypes::{ElementCast};
use dom::bindings::js::JS;
use dom::bindings::utils::ErrorResult;
use dom::document::Document;
use dom::element::{Element, HTMLImageElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, NodeHelpers};
use extra::url::Url;
use servo_util::geometry::to_px;
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use servo_net::image_cache_task;
use servo_util::url::parse_url;
use servo_util::str::DOMString;

use extra::serialize::{Encoder, Encodable};

#[deriving(Encodable)]
pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    extra: Untraceable,
}

struct Untraceable {
    image: Option<Url>,
}

impl<S: Encoder> Encodable<S> for Untraceable {
    fn encode(&self, _s: &mut S) {
    }
}

impl HTMLImageElementDerived for EventTarget {
    fn is_htmlimageelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLImageElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLImageElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, document),
            extra: Untraceable {
                image: None,
            }
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLImageElement> {
        let element = HTMLImageElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLImageElementBinding::Wrap)
    }
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    fn update_image(&mut self, value: Option<DOMString>, url: Option<Url>) {
        let elem = &mut self.htmlelement.element;
        let document = elem.node.owner_doc();
        let window = document.get().window.get();
        let image_cache = &window.image_cache_task;
        match value {
            None => {
                self.extra.image = None;
            }
            Some(src) => {
                let img_url = parse_url(src, url);
                self.extra.image = Some(img_url.clone());

                // inform the image cache to load this, but don't store a
                // handle.
                //
                // TODO (Issue #84): don't prefetch if we are within a
                // <noscript> tag.
                image_cache.send(image_cache_task::Prefetch(img_url));
            }
        }
    }

    pub fn AfterSetAttr(&mut self, name: DOMString, value: DOMString) {
        if "src" == name {
            let document = self.htmlelement.element.node.owner_doc();
            let window = document.get().window.get();
            let url = Some(window.get_url());
            self.update_image(Some(value), url);
        }
    }

    pub fn BeforeRemoveAttr(&mut self, name: DOMString) {
        if "src" == name {
            self.update_image(None, None);
        }
    }

    pub fn Alt(&self) -> DOMString {
        ~""
    }

    pub fn SetAlt(&mut self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self, _abstract_self: &JS<HTMLImageElement>) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, abstract_self: &JS<HTMLImageElement>, src: DOMString) -> ErrorResult {
        let node = &mut self.htmlelement.element;
        node.set_attr(&ElementCast::from(abstract_self), ~"src", src.clone());
        Ok(())
    }

    pub fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> DOMString {
        ~""
    }

    pub fn SetUseMap(&mut self, _use_map: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn IsMap(&self) -> bool {
        false
    }

    pub fn SetIsMap(&self, _is_map: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let node: JS<Node> = NodeCast::from(abstract_self);
        let doc = node.get().owner_doc();
        let page = doc.get().window.get().page;
        let (port, chan) = Chan::new();
        let addr = node.to_trusted_node_address();
        match page.query_layout(ContentBoxQuery(addr, chan), port) {
            ContentBoxResponse(rect) => {
                to_px(rect.size.width) as u32
            }
        }
    }

    pub fn SetWidth(&mut self, abstract_self: &JS<HTMLImageElement>, width: u32) -> ErrorResult {
        let mut elem: JS<Element> = ElementCast::from(abstract_self);
        let mut elem_clone = elem.clone();
        elem.get_mut().set_attr(&mut elem_clone, ~"width", width.to_str());
        Ok(())
    }

    pub fn Height(&self, abstract_self: &JS<HTMLImageElement>) -> u32 {
        let node = &self.htmlelement.element.node;
        let doc = node.owner_doc();
        let page = doc.get().window.get().page;
        let (port, chan) = Chan::new();
        let this_node: JS<Node> = NodeCast::from(abstract_self);
        let addr = this_node.to_trusted_node_address();
        match page.query_layout(ContentBoxQuery(addr, chan), port) {
            ContentBoxResponse(rect) => {
                to_px(rect.size.height) as u32
            }
        }
    }

    pub fn SetHeight(&mut self, abstract_self: &JS<HTMLImageElement>, height: u32) -> ErrorResult {
        let node = &mut self.htmlelement.element;
        node.set_attr(&ElementCast::from(abstract_self), ~"height", height.to_str());
        Ok(())
    }

    pub fn NaturalWidth(&self) -> u32 {
        0
    }

    pub fn NaturalHeight(&self) -> u32 {
        0
    }

    pub fn Complete(&self) -> bool {
        false
    }

    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> DOMString {
        ~""
    }

    pub fn SetLongDesc(&mut self, _longdesc: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        ~""
    }

    pub fn SetBorder(&mut self, _border: DOMString) -> ErrorResult {
        Ok(())
    }
}
