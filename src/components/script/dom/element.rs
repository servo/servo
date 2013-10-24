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

pub struct Element {
    node: Node<ScriptView>,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    attrs: HashMap<~str, ~str>,
    attrs_list: ~[~str], // store an order of attributes.
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
    fn get_local_name<'a>(&'a self) -> &'a str {
        self.tag_name.as_slice()
    }

    fn get_attr<'a>(&'a self, name: &str) -> Option<&'a str> {
        // FIXME: only case-insensitive in the HTML namespace (as opposed to SVG, etc.)
        let name = name.to_ascii_lower();
        let value: Option<&str> = self.attrs.find_equiv(&name).map(|value| {
            let value: &str = *value;
            value
        });

        return value;
    }
}

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str, document: AbstractDocument) -> Element {
        Element {
            node: Node::new(ElementNodeTypeId(type_id), document),
            tag_name: tag_name,
            attrs: HashMap::new(),
            attrs_list: ~[],
            style_attribute: None,
        }
    }

    pub fn set_attr(&mut self,
                    abstract_self: AbstractNode<ScriptView>,
                    raw_name: &DOMString,
                    raw_value: &DOMString) {
        let name = null_str_as_empty(raw_name).to_ascii_lower();
        let value = null_str_as_empty(raw_value);

        // FIXME: reduce the time of `value.clone()`.
        self.attrs.mangle(name.clone(), value.clone(),
                          |new_name: &~str, new_value: ~str| {
                              // register to the ordered list.
                              self.attrs_list.push(new_name.clone());
                              new_value
                          },
                          |_, old_value: &mut ~str, new_value: ~str| {
                              // update value.
                              *old_value = new_value;
                          });

        if "style" == name {
            self.style_attribute = Some(style::parse_style_attribute(
                null_str_as_empty_ref(raw_value)));
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
        Some(self.tag_name.to_owned().to_ascii_upper())
    }

    pub fn Id(&self) -> DOMString {
        None
    }

    pub fn SetId(&self, _id: &DOMString) {
    }

    pub fn GetAttribute(&self, name: &DOMString) -> DOMString {
        self.get_attr(null_str_as_empty_ref(name)).map(|s| s.to_owned())
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
        let (scope, cx) = self.node.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
    }

    pub fn GetElementsByTagNameNS(&self, _namespace: &DOMString, _localname: &DOMString) -> Fallible<@mut HTMLCollection> {
        let (scope, cx) = self.node.get_scope_and_cx();
        Ok(HTMLCollection::new(~[], cx, scope))
    }

    pub fn GetElementsByClassName(&self, _names: &DOMString) -> @mut HTMLCollection {
        let (scope, cx) = self.node.get_scope_and_cx();
        HTMLCollection::new(~[], cx, scope)
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
        let (rects, cx, scope) =
            match win.page.query_layout(ContentBoxesQuery(node, chan), port) {
                ContentBoxesResponse(rects) => {
                    let cx = win.get_cx();
                    let scope = win.reflector().get_jsobject();
                    let rects = do rects.map |r| {
                        ClientRect::new(
                            win,
                            r.origin.y.to_f32().unwrap(),
                            (r.origin.y + r.size.height).to_f32().unwrap(),
                            r.origin.x.to_f32().unwrap(),
                            (r.origin.x + r.size.width).to_f32().unwrap())
                    };
                    (rects, cx, scope)
                },
            };

        ClientRectList::new(rects, cx, scope)
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
