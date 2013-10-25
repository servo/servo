/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::bindings::utils::{Reflectable, DOMString, ErrorResult, Fallible, Reflector};
use dom::bindings::utils::{null_str_as_empty, null_str_as_empty_ref};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::AbstractDocument;
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};
use style;
use servo_util::tree::{TreeNodeRef, ElementLike};

use js::jsapi::{JSContext, JSObject};

use std::comm;
use std::hashmap::HashMap;
use std::ascii::StrAsciiExt;
use servo_util::interning::{intern_string, IntString};
use std::str::eq_slice;

pub struct Element {
    node: Node<ScriptView>,
    tag_name: IntString,     // TODO: This should be an atom, not a ~str.
    id: Option<IntString>,
    classes: ~[IntString],
    attrs: HashMap<IntString, IntString>,
    attrs_list: ~[IntString], // store an order of attributes.
    style_attribute: Option<style::PropertyDeclarationBlock>,
}

impl Reflectable for Element {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!("no wrapping")
    }

    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut Reflectable> {
        self.node.GetParentObject(cx)
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
    HTMLMainElementTypeId,
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
    HTMLTableDataCellElementTypeId,
    HTMLTableHeaderCellElementTypeId,
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

impl ElementLike for Element {
    fn get_local_name<'a>(&'a self) -> &'a IntString {
        &self.tag_name
    }

    fn get_attr<'a>(&'a self, name: &IntString) -> Option<&'a IntString> {
        // FIXME: only case-insensitive in the HTML namespace (as opposed to SVG, etc.)
        let value: Option<&IntString> = self.attrs.find_equiv(name).map(|value| {
            value
        });
        return value;
    }

    fn get_id<'a>(&'a self) -> Option<&'a IntString> {
        match self.id {
            None => None,
            Some(ref id) => Some(id),
        }
    }

    fn get_classes<'a>(&'a self) -> &'a [IntString] {
        let c: &'a [IntString] = self.classes;
        c
    }
}

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str, document: AbstractDocument) -> Element {
        Element {
            node: Node::new(ElementNodeTypeId(type_id), document),
            tag_name: intern_string(tag_name),
            id: None,
            classes: ~[],
            attrs: HashMap::new(),
            attrs_list: ~[],
            style_attribute: None,
        }
    }

    pub fn set_attr(&mut self,
                    abstract_self: AbstractNode<ScriptView>,
                    raw_name: &DOMString,
                    raw_value: &DOMString) {
        static WHITESPACE: &'static [char] = &'static [' ', '\t', '\n', '\r', '\x0C'];

        let name = intern_string(null_str_as_empty(raw_name));
        let value = intern_string(null_str_as_empty(raw_value));

        // FIXME: reduce the time of `value.clone()`
        self.attrs.mangle(name.clone(), value.clone(),
                          |new_name: &IntString, new_value: IntString| {
                              // register to the ordered list.
                              self.attrs_list.push(new_name.clone());
                              new_value
                          },
                          |_, old_value: &mut IntString, new_value: IntString| {
                              // update value.
                              *old_value = new_value;
                          });

        if eq_slice(name.to_ascii_lower(), "id") {
            self.id = Some(value);
        } else if eq_slice(name.to_ascii_lower(), "class") {
            for class in value.to_str_slice().split_iter(WHITESPACE) {
                self.classes.push(intern_string(class));
            }
        }

        if eq_slice("style", name.to_ascii_lower()) {
            self.style_attribute = Some(style::parse_style_attribute(value.to_str_slice()));
        }

        // TODO: update owner document's id hashmap for `document.getElementById()`
        //       if `name` == "id".

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        match abstract_self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                do abstract_self.with_mut_image_element |image| {
                    image.AfterSetAttr(raw_name, raw_value);
                }
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                do abstract_self.with_mut_iframe_element |iframe| {
                    iframe.AfterSetAttr(raw_name, raw_value);
                }
            }
            _ => ()
        }

        if abstract_self.is_in_doc() {
            let document = self.node.owner_doc();
            document.document().content_changed();
        }
    }
}

impl Element {
    pub fn TagName(&self) -> DOMString {
        Some(self.tag_name.to_str_slice().to_ascii_upper())
    }

    pub fn Id(&self) -> DOMString {
        None
    }

    pub fn SetId(&self, _id: &DOMString) {
    }

    pub fn GetAttribute(&self, name: &DOMString) -> DOMString {
        self.get_attr(&intern_string(null_str_as_empty_ref(name))).map(|s| s.to_str())
    }

    pub fn GetAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString) -> DOMString {
        None
    }

    pub fn SetAttribute(&mut self,
                        abstract_self: AbstractNode<ScriptView>,
                        name: &DOMString,
                        value: &DOMString) -> ErrorResult {
        self.set_attr(abstract_self, name, value);
        Ok(())
    }

    pub fn SetAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString, _value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn RemoveAttribute(&self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn RemoveAttributeNS(&self, _namespace: &DOMString, _localname: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HasAttribute(&self, _name: &DOMString) -> bool {
        false
    }

    pub fn HasAttributeNS(&self, _nameapce: &DOMString, _localname: &DOMString) -> bool {
        false
    }

    pub fn GetElementsByTagName(&self, _localname: &DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    pub fn GetElementsByTagNameNS(&self, _namespace: &DOMString, _localname: &DOMString) -> Fallible<@mut HTMLCollection> {
        Ok(HTMLCollection::new(self.node.owner_doc().document().window, ~[]))
    }

    pub fn GetElementsByClassName(&self, _names: &DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    pub fn MozMatchesSelector(&self, _selector: &DOMString) -> Fallible<bool> {
        Ok(false)
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
        let win = self.node.owner_doc().document().window;
        let node = abstract_self;
        assert!(node.is_element());
        let (port, chan) = comm::stream();
        let rects =
            match win.page.query_layout(ContentBoxesQuery(node, chan), port) {
                ContentBoxesResponse(rects) => {
                    do rects.map |r| {
                        ClientRect::new(
                            win,
                            r.origin.y.to_f32().unwrap(),
                            (r.origin.y + r.size.height).to_f32().unwrap(),
                            r.origin.x.to_f32().unwrap(),
                            (r.origin.x + r.size.width).to_f32().unwrap())
                    }
                },
            };

        ClientRectList::new(win, rects)
    }

    pub fn GetBoundingClientRect(&self, abstract_self: AbstractNode<ScriptView>) -> @mut ClientRect {
        let win = self.node.owner_doc().document().window;
        let node = abstract_self;
        assert!(node.is_element());
        let (port, chan) = comm::stream();
        match win.page.query_layout(ContentBoxQuery(node, chan), port) {
            ContentBoxResponse(rect) => {
                ClientRect::new(
                    win,
                    rect.origin.y.to_f32().unwrap(),
                    (rect.origin.y + rect.size.height).to_f32().unwrap(),
                    rect.origin.x.to_f32().unwrap(),
                    (rect.origin.x + rect.size.width).to_f32().unwrap())
            }
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

    pub fn GetInnerHTML(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn SetInnerHTML(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetOuterHTML(&self) -> Fallible<DOMString> {
        Ok(None)
    }

    pub fn SetOuterHTML(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn InsertAdjacentHTML(&mut self, _position: &DOMString, _text: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn QuerySelector(&self, _selectors: &DOMString) -> Fallible<Option<AbstractNode<ScriptView>>> {
        Ok(None)
    }
}

pub struct Attr {
    name: IntString,
    value: IntString,
}

impl Attr {
    pub fn new(name: ~str, value: ~str) -> Attr {
        Attr {
            name: intern_string(name),
            value: intern_string(value)
        }
    }
}
