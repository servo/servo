//
// Element nodes.
//

use dom::node::{ElementNodeTypeId, Node};
use dom::bindings::clientrectlist::ClientRectListImpl;

use core::str::eq_slice;
use core::cell::Cell;
use std::net::url::Url;

pub struct Element {
    parent: Node,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    attrs: ~[Attr],
}

#[deriving_eq]
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

pub struct HTMLImageElement {
    parent: Element,
    image: Option<Url>,
}

//
// Element methods
//

pub impl Element {
    static pub fn new(type_id: ElementTypeId, tag_name: ~str) -> Element {
        Element {
            parent: Node::new(ElementNodeTypeId(type_id)),
            tag_name: tag_name,
            attrs: ~[]
        }
    }

    fn get_attr(&self, name: &str) -> Option<&self/str> {
        // FIXME: Need an each() that links lifetimes in Rust.
        for uint::range(0, self.attrs.len()) |i| {
            if eq_slice(self.attrs[i].name, name) {
                let val: &str = self.attrs[i].value;
                return Some(val);
            }
        }
        return None;
    }

    fn set_attr(&mut self, name: &str, value: ~str) {
        // FIXME: We need a better each_mut in Rust; this is ugly.
        let value_cell = Cell(value);
        for uint::range(0, self.attrs.len()) |i| {
            if eq_slice(self.attrs[i].name, name) {
                self.attrs[i].value = value_cell.take();
                return;
            }
        }
        self.attrs.push(Attr::new(name.to_str(), value_cell.take()));
    }

    fn getClientRects(&self) -> Option<~ClientRectListImpl> {
        Some(~ClientRectListImpl::new())
    }
}

pub struct Attr {
    name: ~str,
    value: ~str,
}

impl Attr {
    static pub fn new(name: ~str, value: ~str) -> Attr {
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

