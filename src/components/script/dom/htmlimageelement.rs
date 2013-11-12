/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLImageElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLImageElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};
use extra::url::Url;
use servo_util::geometry::to_px;
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::url::make_url;
use servo_util::tree::ElementLike;

pub struct HTMLImageElement {
    htmlelement: HTMLElement,
    image: Option<Url>,
}

impl HTMLImageElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLImageElement {
        HTMLImageElement {
            htmlelement: HTMLElement::new_inherited(HTMLImageElementTypeId, localName, document),
            image: None,
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLImageElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLImageElementBinding::Wrap)
    }
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    pub fn update_image(&mut self, image_cache: ImageCacheTask, url: Option<Url>) {
        let elem = &mut self.htmlelement.element;
        let src_opt = elem.get_attr("src").map(|x| x.to_str());
        match src_opt {
            None => {}
            Some(src) => {
                let img_url = make_url(src, url);
                self.image = Some(img_url.clone());

                // inform the image cache to load this, but don't store a
                // handle.
                //
                // TODO (Issue #84): don't prefetch if we are within a
                // <noscript> tag.
                image_cache.send(image_cache_task::Prefetch(img_url));
            }
        }
    }

    pub fn AfterSetAttr(&mut self, name: &DOMString, _value: &DOMString) {
        if "src" == *name {
            let document = self.htmlelement.element.node.owner_doc();
            let window = document.document().window;
            let url = window.page.url.as_ref().map(|&(ref url, _)| url.clone());
            self.update_image(window.image_cache_task.clone(), url);
        }
    }

    pub fn Alt(&self) -> DOMString {
        ~""
    }

    pub fn SetAlt(&mut self, _alt: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self, _abstract_self: AbstractNode<ScriptView>) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self,
                  abstract_self: AbstractNode<ScriptView>,
                  src: &DOMString) -> ErrorResult {
        let node = &mut self.htmlelement.element;
        node.set_attr(abstract_self,
                      &Some(~"src"),
                      &Some(src.clone()));
        Ok(())
    }

    pub fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn UseMap(&self) -> DOMString {
        ~""
    }

    pub fn SetUseMap(&mut self, _use_map: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn IsMap(&self) -> bool {
        false
    }

    pub fn SetIsMap(&self, _is_map: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self, abstract_self: AbstractNode<ScriptView>) -> u32 {
        let node = &self.htmlelement.element.node;
        let page = node.owner_doc().document().window.page;
        let (port, chan) = stream();
        match page.query_layout(ContentBoxQuery(abstract_self, chan), port) {
            ContentBoxResponse(rect) => {
                to_px(rect.size.width) as u32
            }
        }
    }

    pub fn SetWidth(&mut self,
                    abstract_self: AbstractNode<ScriptView>,
                    width: u32) -> ErrorResult {
        let node = &mut self.htmlelement.element;
        node.set_attr(abstract_self,
                      &Some(~"width"),
                      &Some(width.to_str()));
        Ok(())
    }

    pub fn Height(&self, abstract_self: AbstractNode<ScriptView>) -> u32 {
        let node = &self.htmlelement.element.node;
        let page = node.owner_doc().document().window.page;
        let (port, chan) = stream();
        match page.query_layout(ContentBoxQuery(abstract_self, chan), port) {
            ContentBoxResponse(rect) => {
                to_px(rect.size.height) as u32
            }
        }
    }

    pub fn SetHeight(&mut self,
                     abstract_self: AbstractNode<ScriptView>,
                     height: u32) -> ErrorResult {
        let node = &mut self.htmlelement.element;
        node.set_attr(abstract_self,
                      &Some(~"height"),
                      &Some(height.to_str()));
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

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
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

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        ~""
    }

    pub fn SetBorder(&mut self, _border: &DOMString) -> ErrorResult {
        Ok(())
    }
}
