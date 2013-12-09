/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attrlist::AttrList;
use dom::bindings::utils::{Reflectable, DOMString, ErrorResult, Fallible, Reflector};
use dom::bindings::utils::{null_str_as_empty, NamespaceError};
use dom::bindings::utils::{InvalidCharacter, QName, Name, InvalidXMLName, xml_name_type};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::AbstractDocument;
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use dom::attr:: Attr;
use dom::document;
use dom::namespace;
use dom::namespace::Namespace;
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse};
use style;
use servo_util::tree::{TreeNodeRef, ElementLike};

use std::comm;
use std::hashmap::HashMap;
use std::str::{eq, eq_slice};
use std::ascii::StrAsciiExt;

pub struct Element {
    node: Node<ScriptView>,
    tag_name: ~str,     // TODO: This should be an atom, not a ~str.
    namespace: Namespace,
    attrs: HashMap<~str, ~[@mut Attr]>,
    attrs_insert_order: ~[(~str, Namespace)], // store an order of attributes.
    style_attribute: Option<style::PropertyDeclarationBlock>,
    attr_list: Option<@mut AttrList>
}

impl Reflectable for Element {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
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

    fn get_namespace_url<'a>(&'a self) -> &'a str {
        self.namespace.to_str().unwrap_or("")
    }

    fn get_attr(&self, name: &str) -> Option<~str> {
        self.get_attribute(None, name).map(|attr| attr.value.clone())
    }

    fn get_link(&self) -> Option<~str>{
        // FIXME: This is HTML only.
        match self.node.type_id {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId)
            => self.get_attr("href"),
            _ => None,
        }
    }
}

impl<'self> Element {
    pub fn new(type_id: ElementTypeId, tag_name: ~str, namespace: Namespace, document: AbstractDocument) -> Element {
        Element {
            node: Node::new(ElementNodeTypeId(type_id), document),
            tag_name: tag_name,
            namespace: namespace,
            attrs: HashMap::new(),
            attrs_insert_order: ~[],
            attr_list: None,
            style_attribute: None,
        }
    }

    pub fn normalize_attr_name(&self, name: Option<DOMString>) -> ~str {
        //FIXME: Throw for XML-invalid names
        let owner = self.node.owner_doc();
        if owner.document().doctype == document::HTML { // && self.namespace == Namespace::HTML
            null_str_as_empty(&name).to_ascii_lower()
        } else {
            null_str_as_empty(&name)
        }
    }

    pub fn get_attribute(&self,
                         namespace_url: Option<DOMString>,
                         name: &str) -> Option<@mut Attr> {
        let namespace = Namespace::from_str(namespace_url);
        // FIXME: only case-insensitive in the HTML namespace (as opposed to SVG, etc.)
        let name = name.to_ascii_lower();
        self.attrs.find_equiv(&name).and_then(|attrs| {
            do attrs.iter().find |attr| {
                eq_slice(attr.local_name, name) && attr.namespace == namespace
            }.map(|x| *x)
        })
    }

    pub fn set_attr(&mut self,
                    abstract_self: AbstractNode<ScriptView>,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        self.set_attribute(abstract_self, namespace::Null, name, value)
    }

    pub fn set_attribute(&mut self,
                         abstract_self: AbstractNode<ScriptView>,
                         namespace: Namespace,
                         raw_name: DOMString,
                         value: DOMString) -> ErrorResult {
        //FIXME: Throw for XML-invalid names
        //FIXME: Throw for XMLNS-invalid names
        let name = raw_name.to_ascii_lower();
        let (prefix, local_name) = if name.contains(":")  {
            let parts: ~[&str] = name.splitn_iter(':', 1).collect();
            (Some(parts[0].to_owned()), parts[1].to_owned())
        } else {
            (None, name.clone())
        };
        match prefix {
            Some(ref prefix_str) => {
                if (namespace == namespace::Null ||
                    (eq(prefix_str, &~"xml") && namespace != namespace::XML) ||
                    (eq(prefix_str, &~"xmlns") && namespace != namespace::XMLNS)) {
                    return Err(NamespaceError);
                }
            },
            None => {}
        }

        self.node.wait_until_safe_to_modify_dom();

        // FIXME: reduce the time of `value.clone()`.
        let win = self.node.owner_doc().document().window;
        let new_attr = Attr::new_ns(win, local_name.clone(), value.clone(),
                                    name.clone(), namespace.clone(), prefix);
        let mut old_raw_value: Option<DOMString> = None;
        self.attrs.mangle(local_name.clone(), new_attr,
                          |new_name: &~str, new_value: @mut Attr| {
                              // register to the ordered list.
                              let order_value = (new_name.clone(), new_value.namespace.clone());
                              self.attrs_insert_order.push(order_value);
                              ~[new_value]
                          },
                          |name, old_value: &mut ~[@mut Attr], new_value: @mut Attr| {
                              // update value.
                              let mut found = false;
                              for attr in old_value.mut_iter() {
                                  if eq_slice(attr.local_name, *name) &&
                                     attr.namespace == new_value.namespace {
                                      old_raw_value = Some(attr.Value());
                                      *attr = new_value;
                                      found = true;
                                      break;
                                  }

                              }
                              if !found {
                                  old_value.push(new_value);
                                  let order_value = (name.clone(), new_value.namespace.clone());
                                  self.attrs_insert_order.push(order_value);
                              }
                          });

        self.after_set_attr(abstract_self, &namespace, local_name, value, old_raw_value);
        Ok(())
    }

    fn after_set_attr(&mut self,
                      abstract_self: AbstractNode<ScriptView>,
                      namespace: &Namespace,
                      local_name: DOMString,
                      value: DOMString,
                      old_value: Option<DOMString>) {

        match local_name.as_slice() {
            "style" if *namespace == namespace::Null => {
                self.style_attribute = Some(style::parse_style_attribute(value))
            }
            "id" => {
                let doc = self.node.owner_doc();
                let doc = doc.mut_document();
                doc.update_idmap(abstract_self, value.clone(), old_value);
            }
            _ => ()
        }

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        match abstract_self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                do abstract_self.with_mut_image_element |image| {
                    image.AfterSetAttr(local_name.clone(), value.clone());
                }
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                do abstract_self.with_mut_iframe_element |iframe| {
                    iframe.AfterSetAttr(local_name.clone(), value.clone());
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
        self.tag_name.to_ascii_upper()
    }

    pub fn Id(&self, _abstract_self: AbstractNode<ScriptView>) -> DOMString {
        match self.get_attr(&"id") {
            Some(x) => x,
            None => ~""
        }
    }

    pub fn SetId(&mut self, abstract_self: AbstractNode<ScriptView>, id: DOMString) {
        self.set_attribute(abstract_self, namespace::Null, ~"id", id);
    }

    pub fn Attributes(&mut self, abstract_self: AbstractNode<ScriptView>) -> @mut AttrList {
        match self.attr_list {
            None => {
                let window = self.node.owner_doc().document().window;
                let list = AttrList::new(window, abstract_self);
                self.attr_list = Some(list);
                list
            }
            Some(list) => list
        }
    }

    pub fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        self.get_attr(name).map(|s| s.to_owned())
    }

    pub fn GetAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> Option<DOMString> {
        self.get_attribute(namespace, local_name)
            .map(|attr| attr.value.clone())
    }

    pub fn SetAttribute(&mut self,
                        abstract_self: AbstractNode<ScriptView>,
                        name: DOMString,
                        value: DOMString) -> ErrorResult {
        self.set_attr(abstract_self, name, value);
        Ok(())
    }

    pub fn SetAttributeNS(&mut self,
                          abstract_self: AbstractNode<ScriptView>,
                          namespace_url: Option<DOMString>,
                          name: DOMString,
                          value: DOMString) -> ErrorResult {
        let name_type = xml_name_type(name);
        match name_type {
            InvalidXMLName => return Err(InvalidCharacter),
            Name => return Err(NamespaceError),
            QName => {}
        }

        let namespace = Namespace::from_str(namespace_url);
        self.set_attribute(abstract_self, namespace, name, value)
    }

    pub fn RemoveAttribute(&self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn RemoveAttributeNS(&self, _namespace: Option<DOMString>, _localname: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HasAttribute(&self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    pub fn HasAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    pub fn GetElementsByTagName(&self, _localname: DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    pub fn GetElementsByTagNameNS(&self, _namespace: Option<DOMString>, _localname: DOMString) -> Fallible<@mut HTMLCollection> {
        Ok(HTMLCollection::new(self.node.owner_doc().document().window, ~[]))
    }

    pub fn GetElementsByClassName(&self, _names: DOMString) -> @mut HTMLCollection {
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    pub fn MozMatchesSelector(&self, _selector: DOMString) -> Fallible<bool> {
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
        Ok(~"")
    }

    pub fn SetInnerHTML(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetOuterHTML(&self) -> Fallible<DOMString> {
        Ok(~"")
    }

    pub fn SetOuterHTML(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn InsertAdjacentHTML(&mut self, _position: DOMString, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn QuerySelector(&self, _selectors: DOMString) -> Fallible<Option<AbstractNode<ScriptView>>> {
        Ok(None)
    }
}
