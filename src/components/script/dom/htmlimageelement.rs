/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, str, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;
use dom::node::{ScriptView, AbstractNode};
use extra::url::Url;
use layout_interface::{ContentBoxQuery, ContentBoxResponse};
use gfx::geometry::to_px;

pub struct HTMLImageElement {
    parent: HTMLElement,
    image: Option<Url>,
}

impl HTMLImageElement {
    pub fn Alt(&self) -> DOMString {
        null_string
    }

    pub fn SetAlt(&mut self, _alt: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
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
                        unsafe {
                            let page = win.page;
                            let (port, chan) = stream();
                            match (*page).query_layout(ContentBoxQuery(abstract_self, chan), port) {
                                ContentBoxResponse(rect) => {
                                    to_px(rect.size.width) as u32
                                }
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

    pub fn SetWidth(&mut self, width: u32, _rv: &mut ErrorResult) {
        let node = &mut self.parent.parent;
        node.set_attr(&str(~"width"),
                      &str(width.to_str()));
    }

    pub fn Height(&self, abstract_self: AbstractNode<ScriptView>) -> u32 {
        let node = &self.parent.parent.parent;
        match node.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        unsafe {
                            let page = win.page;
                            let (port, chan) = stream();
                            match (*page).query_layout(ContentBoxQuery(abstract_self, chan), port) {
                                ContentBoxResponse(rect) => {
                                    to_px(rect.size.height) as u32
                                }
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

    pub fn SetHeight(&mut self, height: u32, _rv: &mut ErrorResult) {
        let node = &mut self.parent.parent;
        node.set_attr(&str(~"height"),
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
