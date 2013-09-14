/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, str, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;
use dom::node::{ScriptView, AbstractNode};
use extra::url::Url;
use gfx::geometry::to_px;
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use servo_net::image_cache_task;
use servo_net::image_cache_task::ImageCacheTask;
use servo_util::url::make_url;

pub struct HTMLImageElement {
    parent: HTMLElement,
    image: Option<Url>,
}

impl HTMLImageElement {
    /// Makes the local `image` member match the status of the `src` attribute and starts
    /// prefetching the image. This method must be called after `src` is changed.
    pub fn update_image(&mut self, image_cache: ImageCacheTask, url: Option<Url>) {
        let elem = &mut self.parent.parent;
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
        let name = name.to_str();
        if "src" == name {
            let doc = self.parent.parent.parent.owner_doc;
            for doc in doc.iter() {
                do doc.with_base |doc| {
                    for window in doc.window.iter() {
                        let url = window.page.url.map(|&(ref url, _)| url.clone());
                        self.update_image(window.image_cache_task.clone(), url);
                    }
                }
            }
        }
    }

    pub fn Alt(&self) -> DOMString {
        null_string
    }

    pub fn SetAlt(&mut self, _alt: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self, _abstract_self: AbstractNode<ScriptView>) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self,
                  abstract_self: AbstractNode<ScriptView>,
                  src: &DOMString,
                  _rv: &mut ErrorResult) {
        let node = &mut self.parent.parent;
        node.set_attr(abstract_self,
                      &str(~"src"),
                      &str(src.to_str()));
    }

    pub fn CrossOrigin(&self) -> DOMString {
        null_string
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn UseMap(&self) -> DOMString {
        null_string
    }

    pub fn SetUseMap(&mut self, _use_map: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn IsMap(&self) -> bool {
        false
    }

    pub fn SetIsMap(&self, _is_map: bool, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self, abstract_self: AbstractNode<ScriptView>) -> u32 {
        let node = &self.parent.parent.parent;
        match node.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let page = win.page;
                        let (port, chan) = stream();
                        match page.query_layout(ContentBoxQuery(abstract_self, chan), port) {
                            ContentBoxResponse(rect) => {
                                to_px(rect.size.width) as u32
                            }
                        }
                    }
                    None => {
                        debug!("no window");
                        0
                    }
                }
            }
            None => {
                debug!("no document");
                0
            }
        }
    }

    pub fn SetWidth(&mut self,
                    abstract_self: AbstractNode<ScriptView>,
                    width: u32,
                    _rv: &mut ErrorResult) {
        let node = &mut self.parent.parent;
        node.set_attr(abstract_self,
                      &str(~"width"),
                      &str(width.to_str()));
    }

    pub fn Height(&self, abstract_self: AbstractNode<ScriptView>) -> u32 {
        let node = &self.parent.parent.parent;
        match node.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let page = win.page;
                        let (port, chan) = stream();
                        match page.query_layout(ContentBoxQuery(abstract_self, chan), port) {
                            ContentBoxResponse(rect) => {
                                to_px(rect.size.height) as u32
                            }
                        }
                    }
                    None => {
                        debug!("no window");
                        0
                    }
                }
            }
            None => {
                debug!("no document");
                0
            }
        }
    }

    pub fn SetHeight(&mut self,
                     abstract_self: AbstractNode<ScriptView>,
                     height: u32,
                     _rv: &mut ErrorResult) {
        let node = &mut self.parent.parent;
        node.set_attr(abstract_self,
                      &str(~"height"),
                      &str(height.to_str()));
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
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn LongDesc(&self) -> DOMString {
        null_string
    }

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Border(&self) -> DOMString {
        null_string
    }

    pub fn SetBorder(&mut self, _border: &DOMString, _rv: &mut ErrorResult) {
    }
}
