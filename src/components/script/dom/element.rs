/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::bindings::utils::{null_string, str};
use dom::bindings::utils::{BindingObject, CacheableWrapper, DOMString, ErrorResult, WrapperCache};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};
use newcss::stylesheet::Stylesheet;

use js::jsapi::{JSContext, JSObject};

use std::cell::Cell;
use std::comm;
use std::str::eq_slice;
use std::ascii::StrAsciiExt;

pub struct Element {
    parent: Node<ScriptView>,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    attrs: ~[Attr],
    style_attribute: Option<Stylesheet>,
}

impl CacheableWrapper for Element {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!("no wrapping")
    }
}

impl BindingObject for Element {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}

#[deriving(Eq)]
pub enum ElementTypeId {
    HTMLElementTypeId,
    HTMLAnchorElementTypeId,
    HTMLAppletElementTypeId,
    HTMLAreaElementTypeId,
    HTMLAudioElementTypeId,
    HTMLBaseElementTypeId,
    HTMLBRElementTypeId,
    HTMLBodyElementTypeId,
    HTMLButtonElementTypeId,
    HTMLCanvasElementTypeId,
    HTMLDataElementTypeId,
    HTMLDataListElementTypeId,
    HTMLDirectoryElementTypeId,
    HTMLDListElementTypeId,
    HTMLDivElementTypeId,
    HTMLEmbedElementTypeId,
    HTMLFieldSetElementTypeId,
    HTMLFontElementTypeId,
    HTMLFormElementTypeId,
    HTMLFrameElementTypeId,
    HTMLFrameSetElementTypeId,
    HTMLHRElementTypeId,
    HTMLHeadElementTypeId,
    HTMLHeadingElementTypeId,
    HTMLHtmlElementTypeId,
    HTMLIframeElementTypeId,
    HTMLImageElementTypeId,
    HTMLInputElementTypeId,
    HTMLLabelElementTypeId,
    HTMLLegendElementTypeId,
    HTMLLinkElementTypeId,
    HTMLLIElementTypeId,
    HTMLMapElementTypeId,
    HTMLMediaElementTypeId,
    HTMLMetaElementTypeId,
    HTMLMeterElementTypeId,
    HTMLModElementTypeId,
    HTMLObjectElementTypeId,
    HTMLOListElementTypeId,
    HTMLOptGroupElementTypeId,
    HTMLOptionElementTypeId,
    HTMLOutputElementTypeId,
    HTMLParagraphElementTypeId,
    HTMLParamElementTypeId,
    HTMLPreElementTypeId,
    HTMLProgressElementTypeId,
    HTMLQuoteElementTypeId,
    HTMLScriptElementTypeId,
    HTMLSelectElementTypeId,
    HTMLSourceElementTypeId,
    HTMLSpanElementTypeId,
    HTMLStyleElementTypeId,
    HTMLTableElementTypeId,
    HTMLTableCaptionElementTypeId,
    HTMLTableCellElementTypeId,
    HTMLTableColElementTypeId,
    HTMLTableRowElementTypeId,
    HTMLTableSectionElementTypeId,
    HTMLTemplateElementTypeId,
    HTMLTextAreaElementTypeId,
    HTMLTimeElementTypeId,
    HTMLTitleElementTypeId,
    HTMLTrackElementTypeId,
    HTMLUListElementTypeId,
    HTMLVideoElementTypeId,
    HTMLUnknownElementTypeId,
}

//
// Element methods
//

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> Element {
        Element {
            parent: Node::new(ElementNodeTypeId(type_id)),
            tag_name: tag_name,
            attrs: ~[],
            style_attribute: None,
        }
    }

    pub fn get_attr(&'self self, name: &str) -> Option<&'self str> {
        // FIXME: Need an each() that links lifetimes in Rust.
        for attr in self.attrs.iter() {
            if eq_slice(attr.name, name) {
                let val: &str = attr.value;
                return Some(val);
            }
        }
        return None;
    }

    pub fn set_attr(&mut self, raw_name: &DOMString, raw_value: &DOMString) {
        let name = raw_name.to_str();
        let value_cell = Cell::new(raw_value.to_str());
        let mut found = false;
        for attr in self.attrs.mut_iter() {
            if eq_slice(attr.name, name) {
                attr.value = value_cell.take().clone();
                found = true;
                break;
            }
        }
        if !found {
            self.attrs.push(Attr::new(name.to_str(), value_cell.take().clone()));
        }

        if "style" == name {
            self.style_attribute = Some(
                Stylesheet::from_attribute(
                    FromStr::from_str("http://www.example.com/").unwrap(),
                    raw_value.get_ref()));
        }

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        if self.parent.abstract.unwrap().is_iframe_element() {
            do self.parent.abstract.unwrap().with_mut_iframe_element |iframe| {
                iframe.AfterSetAttr(raw_name, raw_value);
            }
        }

        match self.parent.owner_doc {
            Some(owner) => do owner.with_base |owner| { owner.content_changed() },
            None => {}
        }
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.parent.owner_doc.unwrap();
        let win = doc.with_base(|doc| doc.window.unwrap());
        let cx = win.page.js_info.get_ref().js_compartment.cx.ptr;
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }
}

impl Element {
    pub fn TagName(&self) -> DOMString {
        str(self.tag_name.to_owned().to_ascii_upper())
    }

    pub fn Id(&self) -> DOMString {
        null_string
    }

    pub fn SetId(&self, _id: &DOMString) {
    }

    pub fn GetAttribute(&self, name: &DOMString) -> DOMString {
        match self.get_attr(name.get_ref()) {
            Some(val) => str(val.to_owned()),
            None => null_string
        }
    }

    pub fn GetAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString) -> DOMString {
        null_string
    }

    pub fn SetAttribute(&mut self, name: &DOMString, value: &DOMString, _rv: &mut ErrorResult) {
        self.set_attr(name, value);
    }

    pub fn SetAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn RemoveAttribute(&self, _name: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn RemoveAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn HasAttribute(&self, _name: &DOMString) -> bool {
        false
    }

    pub fn HasAttributeNS(&self, _nameapce: &DOMString, _localname: &DOMString) -> bool {
        false
    }

    pub fn GetElementsByTagName(&self, _localname: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByTagNameNS(&self, _namespace: &DOMString, _localname: &DOMString, _rv: &mut ErrorResult) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByClassName(&self, _names: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn MozMatchesSelector(&self, _selector: &DOMString, _rv: &mut ErrorResult) -> bool {
        false
    }

    pub fn SetCapture(&self, _retargetToElement: bool) {
    }

    pub fn ReleaseCapture(&self) {
    }

    pub fn MozRequestFullScreen(&self) {
    }

    pub fn MozRequestPointerLock(&self) {
    }

    pub fn GetClientRects(&self, abstract_self: AbstractNode<ScriptView>) -> @mut ClientRectList {
        let (rects, cx, scope) = match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let node = abstract_self;
                        assert!(node.is_element());
                        let page = win.page;
                        let (port, chan) = comm::stream();
                        match page.query_layout(ContentBoxesQuery(node, chan), port) {
                            ContentBoxesResponse(rects) => {
                                let cx = page.js_info.get_ref().js_compartment.cx.ptr;
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                let rects = do rects.map |r| {
                                    ClientRect::new(
                                         r.origin.y.to_f32(),
                                         (r.origin.y + r.size.height).to_f32(),
                                         r.origin.x.to_f32(),
                                         (r.origin.x + r.size.width).to_f32(),
                                         cx,
                                         scope)
                                };
                                Some((rects, cx, scope))
                            },
                        }
                    }
                    None => {
                        debug!("no window");
                        None
                    }
                }
            }
            None => {
                debug!("no document");
                None
            }
        }.unwrap();

        ClientRectList::new(rects, cx, scope)
    }

    pub fn GetBoundingClientRect(&self, abstract_self: AbstractNode<ScriptView>) -> @mut ClientRect {
        match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let page = win.page;
                        let node = abstract_self;
                        assert!(node.is_element());
                        let (port, chan) = comm::stream();
                        match page.query_layout(ContentBoxQuery(node, chan), port) {
                            ContentBoxResponse(rect) => {
                                let cx = page.js_info.get_ref().js_compartment.cx.ptr;
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                ClientRect::new(
                                    rect.origin.y.to_f32(),
                                    (rect.origin.y + rect.size.height).to_f32(),
                                    rect.origin.x.to_f32(),
                                    (rect.origin.x + rect.size.width).to_f32(),
                                    cx,
                                    scope)
                            }
                        }
                    }
                    None => fail!("no window")
                }
            }
            None => fail!("no document")
        }
    }

    pub fn ScrollIntoView(&self, _top: bool) {
    }

    pub fn ScrollTop(&self) -> i32 {
        0
    }

    pub fn SetScrollTop(&mut self, _scroll_top: i32) {
    }

    pub fn ScrollLeft(&self) -> i32 {
        0
    }

    pub fn SetScrollLeft(&mut self, _scroll_left: i32) {
    }

    pub fn ScrollWidth(&self) -> i32 {
        0
    }

    pub fn ScrollHeight(&self) -> i32 {
        0
    }

    pub fn ClientTop(&self) -> i32 {
        0
    }

    pub fn ClientLeft(&self) -> i32 {
        0
    }

    pub fn ClientWidth(&self) -> i32 {
        0
    }

    pub fn ClientHeight(&self) -> i32 {
        0
    }

    pub fn GetInnerHTML(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetInnerHTML(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetOuterHTML(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }

    pub fn SetOuterHTML(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn InsertAdjacentHTML(&mut self, _position: &DOMString, _text: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn QuerySelector(&self, _selectors: &DOMString, _rv: &mut ErrorResult) -> Option<AbstractNode<ScriptView>> {
        None
    }
}

pub struct Attr {
    name: ~str,
    value: ~str,
}

impl Attr {
    pub fn new(name: ~str, value: ~str) -> Attr {
        Attr {
            name: name,
            value: value
        }
    }
}
