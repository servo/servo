/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::bindings::codegen::{HTMLHeadElementBinding, HTMLHtmlElementBinding};
use dom::bindings::codegen::{HTMLAnchorElementBinding, HTMLDivElementBinding, HTMLSpanElementBinding};
use dom::bindings::codegen::{HTMLImageElementBinding};
use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::bindings::utils::{CacheableWrapper, BindingObject, WrapperCache};
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};
use newcss::stylesheet::Stylesheet;

use js::jsapi::{JSContext, JSObject};

use std::cell::Cell;
use std::comm::ChanOne;
use std::comm;
use std::str::eq_slice;
use extra::net::url::Url;
use geom::size::Size2D;

use servo_msg::constellation_msg::SubpageId;

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
    HTMLAnchorElementTypeId,
    HTMLAsideElementTypeId,
    HTMLBRElementTypeId,
    HTMLBodyElementTypeId,
    HTMLBoldElementTypeId,
    HTMLDivElementTypeId,
    HTMLFontElementTypeId,
    HTMLFormElementTypeId,
    HTMLHRElementTypeId,
    HTMLHeadElementTypeId,
    HTMLHeadingElementTypeId,
    HTMLHtmlElementTypeId,
    HTMLIframeElementTypeId,
    HTMLImageElementTypeId,
    HTMLInputElementTypeId,
    HTMLItalicElementTypeId,
    HTMLLinkElementTypeId,
    HTMLListItemElementTypeId,
    HTMLMetaElementTypeId,
    HTMLOListElementTypeId,
    HTMLOptionElementTypeId,
    HTMLParagraphElementTypeId,
    HTMLScriptElementTypeId,
    HTMLSectionElementTypeId,
    HTMLSelectElementTypeId,
    HTMLSmallElementTypeId,
    HTMLSpanElementTypeId,
    HTMLStyleElementTypeId,
    HTMLTableBodyElementTypeId,
    HTMLTableCellElementTypeId,
    HTMLTableElementTypeId,
    HTMLTableRowElementTypeId,
    HTMLTitleElementTypeId,
    HTMLUListElementTypeId,
    UnknownElementTypeId,
}

//
// Regular old elements
//

pub struct HTMLAsideElement     { parent: HTMLElement }
pub struct HTMLBRElement        { parent: HTMLElement }
pub struct HTMLBodyElement      { parent: HTMLElement }
pub struct HTMLBoldElement      { parent: HTMLElement }
pub struct HTMLDivElement       { parent: HTMLElement }
pub struct HTMLFontElement      { parent: HTMLElement }
pub struct HTMLFormElement      { parent: HTMLElement }
pub struct HTMLHRElement        { parent: HTMLElement }
pub struct HTMLHeadElement      { parent: HTMLElement }
pub struct HTMLHtmlElement      { parent: HTMLElement }
pub struct HTMLInputElement     { parent: HTMLElement }
pub struct HTMLItalicElement    { parent: HTMLElement }
pub struct HTMLLinkElement      { parent: HTMLElement }
pub struct HTMLListItemElement  { parent: HTMLElement }
pub struct HTMLMetaElement      { parent: HTMLElement }
pub struct HTMLOListElement     { parent: HTMLElement }
pub struct HTMLOptionElement    { parent: HTMLElement }
pub struct HTMLParagraphElement { parent: HTMLElement }
pub struct HTMLScriptElement    { parent: HTMLElement }
pub struct HTMLSectionElement   { parent: HTMLElement }
pub struct HTMLSelectElement    { parent: HTMLElement }
pub struct HTMLSmallElement     { parent: HTMLElement }
pub struct HTMLSpanElement      { parent: HTMLElement }
pub struct HTMLStyleElement     { parent: HTMLElement }
pub struct HTMLTableBodyElement { parent: HTMLElement }
pub struct HTMLTableCellElement { parent: HTMLElement }
pub struct HTMLTableElement     { parent: HTMLElement }
pub struct HTMLTableRowElement  { parent: HTMLElement }
pub struct HTMLTitleElement     { parent: HTMLElement }
pub struct HTMLUListElement     { parent: HTMLElement }
pub struct UnknownElement       { parent: HTMLElement }

impl HTMLHtmlElement {
    pub fn Version(&self) -> DOMString {
        null_string
    }

    pub fn SetVersion(&mut self, _version: &DOMString, _rv: &mut ErrorResult) {
    }
}

impl HTMLDivElement {
    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }
}

pub macro_rules! generate_cacheable_wrapper(
    ($name: ident, $wrap: path) => (
        impl CacheableWrapper for $name {
            fn get_wrappercache(&mut self) -> &mut WrapperCache {
                self.parent.get_wrappercache()
            }

            fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
                let mut unused = false;
                $wrap(cx, scope, self, &mut unused)
            }
        }
    )
)

pub macro_rules! generate_binding_object(
    ($name: ident) => (
        impl BindingObject for $name {
            fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
                self.parent.GetParentObject(cx)
            }
        }
    )
)

generate_cacheable_wrapper!(HTMLHeadElement, HTMLHeadElementBinding::Wrap)
generate_binding_object!(HTMLHeadElement)
generate_cacheable_wrapper!(HTMLHtmlElement, HTMLHtmlElementBinding::Wrap)
generate_binding_object!(HTMLHtmlElement)
generate_cacheable_wrapper!(HTMLAnchorElement, HTMLAnchorElementBinding::Wrap)
generate_binding_object!(HTMLAnchorElement)
generate_cacheable_wrapper!(HTMLDivElement, HTMLDivElementBinding::Wrap)
generate_binding_object!(HTMLDivElement)
generate_cacheable_wrapper!(HTMLImageElement, HTMLImageElementBinding::Wrap)
generate_binding_object!(HTMLImageElement)
generate_cacheable_wrapper!(HTMLSpanElement, HTMLSpanElementBinding::Wrap)
generate_binding_object!(HTMLSpanElement)

//
// Fancier elements
//

pub struct HTMLHeadingElement {
    parent: HTMLElement,
    level: HeadingLevel,
}

pub struct HTMLIframeElement {
    parent: HTMLElement,
    frame: Option<Url>,
    subpage_id: Option<SubpageId>,
    size_future_chan: Option<ChanOne<Size2D<uint>>>,
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
        for self.attrs.iter().advance |attr| {
            if eq_slice(attr.name, name) {
                let val: &str = attr.value;
                return Some(val);
            }
        }
        return None;
    }

    pub fn set_attr(&mut self, name: &DOMString, value: &DOMString) {
        let name = name.to_str();
        let value = value.to_str();
        let value_cell = Cell::new(value);
        let mut found = false;
        for self.attrs.mut_iter().advance |attr| {
            if eq_slice(attr.name, name) {
                attr.value = value_cell.take().clone();
                found = true;
                break;
            }
        }
        if !found {
            self.attrs.push(Attr::new(name.to_str(), value_cell.take().clone()));
        }

        match self.parent.owner_doc {
            Some(owner) => do owner.with_base |owner| { owner.content_changed() },
            None => {}
        }
    }

    fn get_scope_and_cx(&self) -> (*JSObject, *JSContext) {
        let doc = self.parent.owner_doc.get();
        let win = doc.with_base(|doc| doc.window.get());
        let cx = unsafe {(*win.page).js_info.get_ref().js_compartment.cx.ptr};
        let cache = win.get_wrappercache();
        let scope = cache.get_wrapper();
        (scope, cx)
    }
}

impl Element {
    pub fn TagName(&self) -> DOMString {
        null_string
    }

    pub fn Id(&self) -> DOMString {
        null_string
    }

    pub fn SetId(&self, _id: &DOMString) {
    }

    pub fn GetAttribute(&self, _name: &DOMString) -> DOMString {
        null_string
    }

    pub fn GetAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString) -> DOMString {
        null_string
    }

    pub fn SetAttribute(&self, _name: &DOMString, _value: &DOMString, _rv: &mut ErrorResult) {
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
                        // TODO(tkuehn): currently just queries top-level page layout. Needs to query
                        // subframe layout if this element is in a subframe. Probably need an ID field.
                        match unsafe {(*page).query_layout(ContentBoxesQuery(node, chan), port)} {
                            Ok(ContentBoxesResponse(rects)) => {
                                let cx = unsafe {(*page).js_info.get_ref().js_compartment.cx.ptr};
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
                            Err(()) => {
                                debug!("layout query error");
                                None
                            }
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
        }.get();

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
                        match unsafe{(*page).query_layout(ContentBoxQuery(node, chan), port)} {
                            Ok(ContentBoxResponse(rect)) => {
                                let cx = unsafe {(*page).js_info.get_ref().js_compartment.cx.ptr};
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                ClientRect::new(
                                    rect.origin.y.to_f32(),
                                    (rect.origin.y + rect.size.height).to_f32(),
                                    rect.origin.x.to_f32(),
                                    (rect.origin.x + rect.size.width).to_f32(),
                                    cx,
                                    scope)
                            },
                            Err(()) => fail!("error querying layout")
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

pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}
