/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::bindings::utils::{DOMString, CacheableWrapper};
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::node::{ElementNodeTypeId, Node, ScriptView};
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};

use std::cell::Cell;
use std::comm::ChanOne;
use std::comm;
use std::str::eq_slice;
use extra::net::url::Url;
use geom::size::Size2D;

pub struct Element {
    parent: Node<ScriptView>,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    attrs: ~[Attr],
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

pub struct HTMLAnchorElement    { parent: Element }
pub struct HTMLAsideElement     { parent: Element }
pub struct HTMLBRElement        { parent: Element }
pub struct HTMLBodyElement      { parent: Element }
pub struct HTMLBoldElement      { parent: Element }
pub struct HTMLDivElement       { parent: Element }
pub struct HTMLFontElement      { parent: Element }
pub struct HTMLFormElement      { parent: Element }
pub struct HTMLHRElement        { parent: Element }
pub struct HTMLHeadElement      { parent: Element }
pub struct HTMLHtmlElement      { parent: Element }
pub struct HTMLInputElement     { parent: Element }
pub struct HTMLItalicElement    { parent: Element }
pub struct HTMLLinkElement      { parent: Element }
pub struct HTMLListItemElement  { parent: Element }
pub struct HTMLMetaElement      { parent: Element }
pub struct HTMLOListElement     { parent: Element }
pub struct HTMLOptionElement    { parent: Element }
pub struct HTMLParagraphElement { parent: Element }
pub struct HTMLScriptElement    { parent: Element }
pub struct HTMLSectionElement   { parent: Element }
pub struct HTMLSelectElement    { parent: Element }
pub struct HTMLSmallElement     { parent: Element }
pub struct HTMLSpanElement      { parent: Element }
pub struct HTMLStyleElement     { parent: Element }
pub struct HTMLTableBodyElement { parent: Element }
pub struct HTMLTableCellElement { parent: Element }
pub struct HTMLTableElement     { parent: Element }
pub struct HTMLTableRowElement  { parent: Element }
pub struct HTMLTitleElement     { parent: Element }
pub struct HTMLUListElement     { parent: Element }
pub struct UnknownElement       { parent: Element }

//
// Fancier elements
//

pub struct HTMLHeadingElement {
    parent: Element,
    level: HeadingLevel,
}

pub struct HTMLIframeElement {
    parent: Element,
    frame: Option<Url>,
    size_future_chan: Option<ChanOne<Size2D<uint>>>,
}

pub struct HTMLImageElement {
    parent: Element,
    image: Option<Url>,
}

//
// Element methods
//

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str) -> Element {
        Element {
            parent: Node::new(ElementNodeTypeId(type_id)),
            tag_name: tag_name,
            attrs: ~[]
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

    pub fn getClientRects(&self) -> Option<@mut ClientRectList> {
        let (rects, cx, scope) = match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let node = self.parent.abstract.get();
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
        
        Some(ClientRectList::new(rects, cx, scope))
    }

    pub fn getBoundingClientRect(&self) -> Option<@mut ClientRect> {
        match self.parent.owner_doc {
            Some(doc) => {
                match doc.with_base(|doc| doc.window) {
                    Some(win) => {
                        let page = win.page;
                        let node = self.parent.abstract.get();
                        assert!(node.is_element());
                        let (port, chan) = comm::stream();
                        match unsafe{(*page).query_layout(ContentBoxQuery(node, chan), port)} {
                            Ok(ContentBoxResponse(rect)) => {
                                let cx = unsafe {(*page).js_info.get_ref().js_compartment.cx.ptr};
                                let cache = win.get_wrappercache();
                                let scope = cache.get_wrapper();
                                Some(ClientRect::new(
                                         rect.origin.y.to_f32(),
                                         (rect.origin.y + rect.size.height).to_f32(),
                                         rect.origin.x.to_f32(),
                                         (rect.origin.x + rect.size.width).to_f32(),
                                         cx,
                                         scope))
                            },
                            Err(()) => {
                                debug!("error querying layout");
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
        }
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

