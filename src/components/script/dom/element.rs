/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::Attr;
use dom::attrlist::AttrList;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::utils::{ErrorResult, Fallible, NamespaceError, InvalidCharacter};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::AbstractDocument;
use dom::node::{AbstractNode, ElementNodeTypeId, Node, NodeIterator};
use dom::document;
use dom::htmlserializer::serialize;
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse, ContentChangedDocumentDamage};
use layout_interface::{MatchSelectorsDocumentDamage};
use style;
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref};

use std::ascii::StrAsciiExt;
use std::cast;
use std::unstable::raw::Box;

pub struct Element {
    node: Node,
    tag_name: DOMString,     // TODO: This should be an atom, not a DOMString.
    namespace: Namespace,
    attrs: ~[@mut Attr],
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


impl Element {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: ~str, namespace: Namespace, document: AbstractDocument) -> Element {
        Element {
            node: Node::new_inherited(ElementNodeTypeId(type_id), document),
            tag_name: tag_name,
            namespace: namespace,
            attrs: ~[],
            attr_list: None,
            style_attribute: None,
        }
    }

    pub fn html_element_in_html_document(&self) -> bool {
        let owner = self.node.owner_doc();
        self.namespace == namespace::HTML &&
        // FIXME: check that this matches what the spec calls "is in an HTML document"
        owner.document().doctype == document::HTML
    }

    pub fn get_attribute(&self,
                         namespace: Namespace,
                         name: &str) -> Option<@mut Attr> {
        self.attrs.iter().find(|attr| {
            name == attr.local_name && attr.namespace == namespace
        }).map(|&x| x)
    }

    #[inline]
    pub unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str)
                                          -> Option<&'static str> {
        self.attrs.iter().find(|attr: & &@mut Attr| {
            // unsafely avoid a borrow because this is accessed by many tasks
            // during parallel layout
            let attr: ***Box<Attr> = cast::transmute(attr);
            name == (***attr).data.local_name && (***attr).data.namespace == *namespace
       }).map(|attr| {
            let attr: **Box<Attr> = cast::transmute(attr);
            cast::transmute((**attr).data.value.as_slice())
        })
    }

    pub fn set_attr(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString)
                    -> ErrorResult {
        self.set_attribute(abstract_self, namespace::Null, name, value)
    }

    pub fn set_attribute(&mut self,
                         abstract_self: AbstractNode,
                         namespace: Namespace,
                         name: DOMString,
                         value: DOMString) -> ErrorResult {
        let (prefix, local_name) = get_attribute_parts(name.clone());
        match prefix {
            Some(ref prefix_str) => {
                if (namespace == namespace::Null ||
                    ("xml" == prefix_str.as_slice() && namespace != namespace::XML) ||
                    ("xmlns" == prefix_str.as_slice() && namespace != namespace::XMLNS)) {
                    return Err(NamespaceError);
                }
            },
            None => {}
        }

        self.node.wait_until_safe_to_modify_dom();

        // FIXME: reduce the time of `value.clone()`.
        let mut old_raw_value: Option<DOMString> = None;
        for attr in self.attrs.iter() {
            if attr.local_name == local_name {
                old_raw_value = Some(attr.set_value(value.clone()));
                break;
            }
        }

        if old_raw_value.is_none() {
            let win = self.node.owner_doc().document().window;
            let new_attr = Attr::new_ns(win, local_name.clone(), value.clone(),
                                        name.clone(), namespace.clone(),
                                        prefix);
            self.attrs.push(new_attr);
        }

        if namespace == namespace::Null {
            self.after_set_attr(abstract_self, local_name, value, old_raw_value);
        }
        Ok(())
    }

    fn after_set_attr(&mut self,
                      abstract_self: AbstractNode,
                      local_name: DOMString,
                      value: DOMString,
                      old_value: Option<DOMString>) {

        match local_name.as_slice() {
            "style" => {
                let doc = self.node.owner_doc();
                let base_url = doc.document().url.clone();
                self.style_attribute = Some(style::parse_style_attribute(value, &base_url))
            }
            "id" if abstract_self.is_in_doc() => {
                // XXX: this dual declaration are workaround to avoid the compile error:
                // "borrowed value does not live long enough"
                let doc = self.node.owner_doc();
                let doc = doc.mut_document();
                doc.update_idmap(abstract_self, Some(value.clone()), old_value);
            }
            _ => ()
        }

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        match abstract_self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                abstract_self.with_mut_image_element(|image| {
                    image.AfterSetAttr(local_name.clone(), value.clone());
                });
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                abstract_self.with_mut_iframe_element(|iframe| {
                    iframe.AfterSetAttr(local_name.clone(), value.clone());
                });
            }
            _ => ()
        }

        self.notify_attribute_changed(abstract_self, local_name);
    }

    pub fn remove_attribute(&mut self,
                            abstract_self: AbstractNode,
                            namespace: Namespace,
                            name: DOMString) -> ErrorResult {
        let (_, local_name) = get_attribute_parts(name.clone());

        self.node.wait_until_safe_to_modify_dom();

        let idx = self.attrs.iter().position(|attr: &@mut Attr| -> bool {
            attr.local_name == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                let removed = self.attrs.remove(idx);
                let removed_raw_value = Some(removed.Value());

                if namespace == namespace::Null {
                    self.after_remove_attr(abstract_self, local_name, removed_raw_value);
                }
            }
        };

        Ok(())
    }

    fn after_remove_attr(&mut self,
                         abstract_self: AbstractNode,
                         local_name: DOMString,
                         old_value: Option<DOMString>) {
        match local_name.as_slice() {
            "style" => {
                self.style_attribute = None
            }
            "id" if abstract_self.is_in_doc() => {
                // XXX: this dual declaration are workaround to avoid the compile error:
                // "borrowed value does not live long enough"
                let doc = self.node.owner_doc();
                let doc = doc.mut_document();
                doc.update_idmap(abstract_self, None, old_value);
            }
            _ => ()
        }

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        match abstract_self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                abstract_self.with_mut_image_element(|image| {
                    image.AfterRemoveAttr(local_name.clone());
                });
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                abstract_self.with_mut_iframe_element(|iframe| {
                    iframe.AfterRemoveAttr(local_name.clone());
                });
            }
            _ => ()
        }

        self.notify_attribute_changed(abstract_self, local_name);
    }

    fn notify_attribute_changed(&self,
                                abstract_self: AbstractNode,
                                local_name: DOMString) {
        if abstract_self.is_in_doc() {
            let damage = match local_name.as_slice() {
                "style" | "id" | "class" => MatchSelectorsDocumentDamage,
                _ => ContentChangedDocumentDamage
            };
            let document = self.node.owner_doc();
            document.document().damage_and_reflow(damage);
        }
    }

    pub fn is_void(&self) -> bool {
        if self.namespace != namespace::HTML {
            return false
        }
        match self.tag_name.as_slice() {
            /* List of void elements from
            http://www.whatwg.org/specs/web-apps/current-work/multipage/the-end.html#html-fragment-serialization-algorithm */
            "area" | "base" | "basefont" | "bgsound" | "br" | "col" | "embed" |
            "frame" | "hr" | "img" | "input" | "keygen" | "link" | "menuitem" |
            "meta" | "param" | "source" | "track" | "wbr" => true,
            _ => false
        }
    }
}

// http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
impl Element {
    pub fn get_url_attribute(&self, name: &str) -> DOMString {
        // XXX Resolve URL.
        self.get_string_attribute(name)
    }
    pub fn set_url_attribute(&mut self, abstract_self: AbstractNode,
                             name: &str, value: DOMString) {
        self.set_string_attribute(abstract_self, name, value);
    }

    pub fn get_string_attribute(&self, name: &str) -> DOMString {
        match self.get_attribute(Null, name) {
            Some(x) => x.Value(),
            None => ~""
        }
    }
    pub fn set_string_attribute(&mut self, abstract_self: AbstractNode,
                                name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower());
        self.set_attribute(abstract_self, Null, name.to_owned(), value);
    }
}

impl Element {
    // http://dom.spec.whatwg.org/#dom-element-tagname
    pub fn TagName(&self) -> DOMString {
        self.tag_name.to_ascii_upper()
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    pub fn Id(&self, _abstract_self: AbstractNode) -> DOMString {
        self.get_string_attribute("id")
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    pub fn SetId(&mut self, abstract_self: AbstractNode, id: DOMString) {
        self.set_string_attribute(abstract_self, "id", id);
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    pub fn Attributes(&mut self, abstract_self: AbstractNode) -> @mut AttrList {
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

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    pub fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        self.get_attribute(Null, name).map(|s| s.Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    pub fn GetAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> Option<DOMString> {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.get_attribute(namespace, local_name)
            .map(|attr| attr.value.clone())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    pub fn SetAttribute(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString)
                        -> ErrorResult {
        // FIXME: If name does not match the Name production in XML, throw an "InvalidCharacterError" exception.
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        self.set_attr(abstract_self, name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    pub fn SetAttributeNS(&mut self,
                          abstract_self: AbstractNode,
                          namespace_url: Option<DOMString>,
                          name: DOMString,
                          value: DOMString) -> ErrorResult {
        let name_type = xml_name_type(name);
        match name_type {
            InvalidXMLName => return Err(InvalidCharacter),
            Name => return Err(NamespaceError),
            QName => {}
        }

        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace_url));
        self.set_attribute(abstract_self, namespace, name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    pub fn RemoveAttribute(&mut self,
                           abstract_self: AbstractNode,
                           name: DOMString) -> ErrorResult {
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        self.remove_attribute(abstract_self, namespace::Null, name)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    pub fn RemoveAttributeNS(&mut self,
                             abstract_self: AbstractNode,
                             namespace: Option<DOMString>,
                             localname: DOMString) -> ErrorResult {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.remove_attribute(abstract_self, namespace, localname)
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    pub fn HasAttribute(&self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    pub fn HasAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    pub fn GetElementsByTagName(&self, _localname: DOMString) -> @mut HTMLCollection {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1660
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    // http://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    pub fn GetElementsByTagNameNS(&self, _namespace: Option<DOMString>, _localname: DOMString) -> Fallible<@mut HTMLCollection> {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1660
        Ok(HTMLCollection::new(self.node.owner_doc().document().window, ~[]))
    }

    // http://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    pub fn GetElementsByClassName(&self, _names: DOMString) -> @mut HTMLCollection {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1660
        HTMLCollection::new(self.node.owner_doc().document().window, ~[])
    }

    // http://dom.spec.whatwg.org/#dom-element-matches
    pub fn MozMatchesSelector(&self, _selector: DOMString) -> Fallible<bool> {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1660
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

    pub fn GetClientRects(&self, abstract_self: AbstractNode) -> @mut ClientRectList {
        let win = self.node.owner_doc().document().window;
        let node = abstract_self;
        assert!(node.is_element());
        let (port, chan) = Chan::new();
        let rects =
            match win.page.query_layout(ContentBoxesQuery(node, chan), port) {
                ContentBoxesResponse(rects) => {
                    rects.map(|r| {
                        ClientRect::new(
                            win,
                            r.origin.y,
                            r.origin.y + r.size.height,
                            r.origin.x,
                            r.origin.x + r.size.width)
                    })
                },
            };

        ClientRectList::new(win, rects)
    }

    pub fn GetBoundingClientRect(&self, abstract_self: AbstractNode) -> @mut ClientRect {
        let win = self.node.owner_doc().document().window;
        let node = abstract_self;
        assert!(node.is_element());
        let (port, chan) = Chan::new();
        match win.page.query_layout(ContentBoxQuery(node, chan), port) {
            ContentBoxResponse(rect) => {
                ClientRect::new(
                    win,
                    rect.origin.y,
                    rect.origin.y + rect.size.height,
                    rect.origin.x,
                    rect.origin.x + rect.size.width)
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

    pub fn GetInnerHTML(&self, abstract_self: AbstractNode) -> Fallible<DOMString> {
        //XXX TODO: XML case
        Ok(serialize(&mut NodeIterator::new(abstract_self, false, false)))
    }

    pub fn SetInnerHTML(&mut self, _abstract_self: AbstractNode, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn GetOuterHTML(&self, abstract_self:AbstractNode) -> Fallible<DOMString> {
        Ok(serialize(&mut NodeIterator::new(abstract_self, true, false)))
    }

    pub fn SetOuterHTML(&mut self, _abstract_self: AbstractNode, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn InsertAdjacentHTML(&mut self, _position: DOMString, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn QuerySelector(&self, _selectors: DOMString) -> Fallible<Option<AbstractNode>> {
        Ok(None)
    }
}

fn get_attribute_parts(name: DOMString) -> (Option<~str>, ~str) {
    //FIXME: Throw for XML-invalid names
    //FIXME: Throw for XMLNS-invalid names
    let (prefix, local_name) = if name.contains(":")  {
        let parts: ~[&str] = name.splitn(':', 1).collect();
        (Some(parts[0].to_owned()), parts[1].to_owned())
    } else {
        (None, name)
    };

    (prefix, local_name)
}
