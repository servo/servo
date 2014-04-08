/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::Attr;
use dom::attrlist::AttrList;
use dom::bindings::codegen::ElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementDerived, HTMLImageElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLIFrameElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{ErrorResult, Fallible, NamespaceError, InvalidCharacter};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::htmlcollection::HTMLCollection;
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::node::{ElementNodeTypeId, Node, NodeHelpers, NodeIterator, document_from_node};
use dom::htmlserializer::serialize;
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery};
use layout_interface::{ContentBoxesResponse, ContentChangedDocumentDamage};
use layout_interface::{MatchSelectorsDocumentDamage};
use style;
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref, split_html_space_chars};

use std::ascii::StrAsciiExt;
use std::cast;

#[deriving(Encodable)]
pub struct Element {
    node: Node,
    tag_name: DOMString,     // TODO: This should be an atom, not a DOMString.
    namespace: Namespace,
    attrs: ~[JS<Attr>],
    style_attribute: Option<style::PropertyDeclarationBlock>,
    attr_list: Option<JS<AttrList>>
}

impl ElementDerived for EventTarget {
    fn is_element(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(_)) => true,
            _ => false
        }
    }
}

impl Reflectable for Element {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.node.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.node.mut_reflector()
    }
}

#[deriving(Eq,Encodable)]
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
    HTMLIFrameElementTypeId,
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

    ElementTypeId,
}

//
// Element methods
//

impl Element {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, namespace: Namespace, document: JS<Document>) -> Element {
        Element {
            node: Node::new_inherited(ElementNodeTypeId(type_id), document),
            tag_name: tag_name,
            namespace: namespace,
            attrs: ~[],
            attr_list: None,
            style_attribute: None,
        }
    }

    pub fn new(tag_name: DOMString, namespace: Namespace, document: &JS<Document>) -> JS<Element> {
        let element = Element::new_inherited(ElementTypeId, tag_name, namespace, document.clone());
        Node::reflect_node(~element, document, ElementBinding::Wrap)
    }

    pub fn html_element_in_html_document(&self) -> bool {
        self.namespace == namespace::HTML &&
        self.node.owner_doc().get().is_html_document
    }
}

impl Element {
    pub unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if self.namespace != namespace::HTML {
            return false
        }
        let owner_doc: *JS<Document> = self.node.owner_doc();
        let owner_doc: **Document = owner_doc as **Document;
        (**owner_doc).is_html_document
    }

    #[inline]
    pub unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str)
                                          -> Option<&'static str> {
        self.attrs.iter().find(|attr: & &JS<Attr>| {
            // unsafely avoid a borrow because this is accessed by many tasks
            // during parallel layout
            let attr: ***Attr = cast::transmute(attr);
            name == (***attr).local_name && (***attr).namespace == *namespace
       }).map(|attr| {
            let attr: **Attr = cast::transmute(attr);
            cast::transmute((**attr).value.as_slice())
        })
    }
}

pub trait AttributeHandlers {
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<JS<Attr>>;
    fn set_attr(&mut self, name: DOMString, value: DOMString) -> ErrorResult;
    fn set_attribute(&mut self, namespace: Namespace, name: DOMString,
                     value: DOMString) -> ErrorResult;
    fn after_set_attr(&mut self, local_name: DOMString, value: DOMString);
    fn remove_attribute(&mut self, namespace: Namespace, name: DOMString) -> ErrorResult;
    fn before_remove_attr(&mut self, local_name: DOMString, old_value: DOMString);
    fn notify_attribute_changed(&self, local_name: DOMString);
    fn has_class(&self, name: &str) -> bool;

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn get_url_attribute(&self, name: &str) -> DOMString;
    fn set_url_attribute(&mut self, name: &str, value: DOMString);
    fn get_string_attribute(&self, name: &str) -> DOMString;
    fn set_string_attribute(&mut self, name: &str, value: DOMString);
    fn set_uint_attribute(&mut self, name: &str, value: u32);
}

pub trait AfterSetAttrListener {
    fn AfterSetAttr(&mut self, name: DOMString, value: DOMString);
}

pub trait BeforeRemoveAttrListener {
    fn BeforeRemoveAttr(&mut self, name: DOMString);
}

impl AttributeHandlers for JS<Element> {
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<JS<Attr>> {
        if self.get().html_element_in_html_document() {
            self.get().attrs.iter().find(|attr| {
                let attr = attr.get();
                name.to_ascii_lower() == attr.local_name && attr.namespace == namespace
            }).map(|x| x.clone())
        } else {
            self.get().attrs.iter().find(|attr| {
                let attr = attr.get();
                name == attr.local_name && attr.namespace == namespace
            }).map(|x| x.clone())
        }
    }

    fn set_attr(&mut self, name: DOMString, value: DOMString) -> ErrorResult {
        self.set_attribute(namespace::Null, name, value)
    }

    fn set_attribute(&mut self, namespace: Namespace, name: DOMString,
                     value: DOMString) -> ErrorResult {
        let (prefix, local_name) = get_attribute_parts(name.clone());
        match prefix {
            Some(ref prefix_str) => {
                if namespace == namespace::Null ||
                   ("xml" == prefix_str.as_slice() && namespace != namespace::XML) ||
                   ("xmlns" == prefix_str.as_slice() && namespace != namespace::XMLNS) {
                    return Err(NamespaceError);
                }
            },
            None => {}
        }

        let node: JS<Node> = NodeCast::from(self);
        node.get().wait_until_safe_to_modify_dom();

        // FIXME: reduce the time of `value.clone()`.
        let idx = self.get().attrs.iter().position(|attr| {
            if self.get().html_element_in_html_document() {
                attr.get().local_name.eq_ignore_ascii_case(local_name)
            } else {
                attr.get().local_name == local_name
            }
        });

        match idx {
            Some(idx) => {
                if namespace == namespace::Null {
                    let old_value = self.get().attrs[idx].get().Value();
                    self.before_remove_attr(local_name.clone(), old_value);
                }
                self.get_mut().attrs[idx].get_mut().set_value(value.clone());
            }
            None => {
                let node: JS<Node> = NodeCast::from(self);
                let doc = node.get().owner_doc().get();
                let new_attr = Attr::new_ns(&doc.window, local_name.clone(), value.clone(),
                                            name.clone(), namespace.clone(),
                                            prefix);
                self.get_mut().attrs.push(new_attr);
            }
        }

        if namespace == namespace::Null {
            self.after_set_attr(local_name, value);
        }
        Ok(())
    }

    fn after_set_attr(&mut self, local_name: DOMString, value: DOMString) {
        let node: JS<Node> = NodeCast::from(self);
        match local_name.as_slice() {
            "style" => {
                let doc = node.get().owner_doc();
                let base_url = doc.get().url().clone();
                self.get_mut().style_attribute = Some(style::parse_style_attribute(value, &base_url))
            }
            "id" if node.is_in_doc() => {
                // XXX: this dual declaration are workaround to avoid the compile error:
                // "borrowed value does not live long enough"
                let mut doc = node.get().owner_doc().clone();
                let doc = doc.get_mut();
                doc.register_named_element(self, value.clone());
            }
            _ => ()
        }

        //XXXjdm We really need something like a vtable so we can call AfterSetAttr.
        //       This hardcoding is awful.
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let mut elem: JS<HTMLImageElement> = HTMLImageElementCast::to(self).unwrap();
                elem.AfterSetAttr(local_name.clone(), value.clone());
            }
            ElementNodeTypeId(HTMLIFrameElementTypeId) => {
                let mut elem: JS<HTMLIFrameElement> = HTMLIFrameElementCast::to(self).unwrap();
                elem.AfterSetAttr(local_name.clone(), value.clone());
            }
            ElementNodeTypeId(HTMLObjectElementTypeId) => {
                let mut elem: JS<HTMLObjectElement> = HTMLObjectElementCast::to(self).unwrap();
                elem.AfterSetAttr(local_name.clone(), value.clone());
            }
            _ => ()
        }

        self.notify_attribute_changed(local_name);
    }

    fn remove_attribute(&mut self, namespace: Namespace, name: DOMString) -> ErrorResult {
        let (_, local_name) = get_attribute_parts(name.clone());

        let node: JS<Node> = NodeCast::from(self);
        node.get().wait_until_safe_to_modify_dom();

        let idx = self.get().attrs.iter().position(|attr| {
            attr.get().local_name == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                if namespace == namespace::Null {
                    let removed_raw_value = self.get().attrs[idx].get().Value();
                    self.before_remove_attr(local_name, removed_raw_value);
                }

                self.get_mut().attrs.remove(idx);
            }
        };

        Ok(())
    }

    fn before_remove_attr(&mut self, local_name: DOMString, old_value: DOMString) {
        let node: JS<Node> = NodeCast::from(self);
        match local_name.as_slice() {
            "style" => {
                self.get_mut().style_attribute = None
            }
            "id" if node.is_in_doc() => {
                // XXX: this dual declaration are workaround to avoid the compile error:
                // "borrowed value does not live long enough"
                let mut doc = node.get().owner_doc().clone();
                let doc = doc.get_mut();
                doc.unregister_named_element(self, old_value);
            }
            _ => ()
        }

        //XXXjdm We really need something like a vtable so we can call BeforeRemoveAttr.
        //       This hardcoding is awful.
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let mut elem: JS<HTMLImageElement> = HTMLImageElementCast::to(self).unwrap();
                elem.BeforeRemoveAttr(local_name.clone());
            }
            ElementNodeTypeId(HTMLIFrameElementTypeId) => {
                let mut elem: JS<HTMLIFrameElement> = HTMLIFrameElementCast::to(self).unwrap();
                elem.BeforeRemoveAttr(local_name.clone());
            }
            _ => ()
        }

        self.notify_attribute_changed(local_name);
    }

    fn notify_attribute_changed(&self, local_name: DOMString) {
        let node: JS<Node> = NodeCast::from(self);
        if node.is_in_doc() {
            let damage = match local_name.as_slice() {
                "style" | "id" | "class" => MatchSelectorsDocumentDamage,
                _ => ContentChangedDocumentDamage
            };
            let document = node.get().owner_doc();
            document.get().damage_and_reflow(damage);
        }
    }

    fn has_class(&self, name: &str) -> bool {
        let class_names = self.get_string_attribute("class");
        let mut classes = split_html_space_chars(class_names);
        classes.any(|class| name == class)
    }

    fn get_url_attribute(&self, name: &str) -> DOMString {
        // XXX Resolve URL.
        self.get_string_attribute(name)
    }
    fn set_url_attribute(&mut self, name: &str, value: DOMString) {
        self.set_string_attribute(name, value);
    }

    fn get_string_attribute(&self, name: &str) -> DOMString {
        match self.get_attribute(Null, name) {
            Some(x) => x.get().Value(),
            None => ~""
        }
    }
    fn set_string_attribute(&mut self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower());
        assert!(self.set_attribute(Null, name.to_owned(), value).is_ok());
    }

    fn set_uint_attribute(&mut self, name: &str, value: u32) {
        assert!(name == name.to_ascii_lower());
        assert!(self.set_attribute(Null, name.to_owned(), value.to_str()).is_ok());
    }
}

impl Element {
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

impl Element {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    pub fn NamespaceURI(&self) -> DOMString {
        self.namespace.to_str().to_owned()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    pub fn TagName(&self) -> DOMString {
        self.tag_name.to_ascii_upper()
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    pub fn Id(&self, abstract_self: &JS<Element>) -> DOMString {
        abstract_self.get_string_attribute("id")
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    pub fn SetId(&mut self, abstract_self: &mut JS<Element>, id: DOMString) {
        abstract_self.set_string_attribute("id", id);
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    pub fn ClassName(&self, abstract_self: &JS<Element>) -> DOMString {
        abstract_self.get_string_attribute("class")
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    pub fn SetClassName(&self, abstract_self: &mut JS<Element>, class: DOMString) {
        abstract_self.set_string_attribute("class", class);
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    pub fn Attributes(&mut self, abstract_self: &JS<Element>) -> JS<AttrList> {
        match self.attr_list {
            None => {
                let doc = self.node.owner_doc();
                let doc = doc.get();
                let list = AttrList::new(&doc.window, abstract_self);
                self.attr_list = Some(list.clone());
                list
            }
            Some(ref list) => list.clone()
        }
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    pub fn GetAttribute(&self, abstract_self: &JS<Element>, name: DOMString) -> Option<DOMString> {
        let name = if abstract_self.get().html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        abstract_self.get_attribute(Null, name).map(|s| s.get().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    pub fn GetAttributeNS(&self, abstract_self: &JS<Element>,
                          namespace: Option<DOMString>,
                          local_name: DOMString) -> Option<DOMString> {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        abstract_self.get_attribute(namespace, local_name)
                     .map(|attr| attr.get().value.clone())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    pub fn SetAttribute(&mut self, abstract_self: &mut JS<Element>,
                        name: DOMString,
                        value: DOMString) -> ErrorResult {
        // FIXME: If name does not match the Name production in XML, throw an "InvalidCharacterError" exception.
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        abstract_self.set_attr(name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    pub fn SetAttributeNS(&mut self,
                          abstract_self: &mut JS<Element>,
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
        abstract_self.set_attribute(namespace, name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    pub fn RemoveAttribute(&mut self,
                           abstract_self: &mut JS<Element>,
                           name: DOMString) -> ErrorResult {
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        abstract_self.remove_attribute(namespace::Null, name)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    pub fn RemoveAttributeNS(&mut self,
                             abstract_self: &mut JS<Element>,
                             namespace: Option<DOMString>,
                             localname: DOMString) -> ErrorResult {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        abstract_self.remove_attribute(namespace, localname)
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    pub fn HasAttribute(&self, abstract_self: &JS<Element>,
                        name: DOMString) -> bool {
        self.GetAttribute(abstract_self, name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    pub fn HasAttributeNS(&self, abstract_self: &JS<Element>,
                          namespace: Option<DOMString>,
                          local_name: DOMString) -> bool {
        self.GetAttributeNS(abstract_self, namespace, local_name).is_some()
    }

    pub fn GetElementsByTagName(&self, abstract_self: &JS<Element>, localname: DOMString) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::by_tag_name(&doc.window, &NodeCast::from(abstract_self), localname)
    }

    pub fn GetElementsByTagNameNS(&self, abstract_self: &JS<Element>, maybe_ns: Option<DOMString>,
                                  localname: DOMString) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace),
            None => Null
        };
        HTMLCollection::by_tag_name_ns(&doc.window, &NodeCast::from(abstract_self), localname, namespace)
    }

    pub fn GetElementsByClassName(&self, abstract_self: &JS<Element>, classes: DOMString) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::by_class_name(&doc.window, &NodeCast::from(abstract_self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    pub fn GetClientRects(&self, abstract_self: &JS<Element>) -> JS<ClientRectList> {
        let doc = self.node.owner_doc();
        let win = &doc.get().window;
        let node: JS<Node> = NodeCast::from(abstract_self);
        let (chan, port) = channel();
        let addr = node.to_trusted_node_address();
        let ContentBoxesResponse(rects) = win.get().page().query_layout(ContentBoxesQuery(addr, chan), port);
        let rects = rects.map(|r| {
            ClientRect::new(
                win,
                r.origin.y,
                r.origin.y + r.size.height,
                r.origin.x,
                r.origin.x + r.size.width)
        });

        ClientRectList::new(win, rects)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getboundingclientrect
    pub fn GetBoundingClientRect(&self, abstract_self: &JS<Element>) -> JS<ClientRect> {
        let doc = self.node.owner_doc();
        let win = &doc.get().window;
        let node: JS<Node> = NodeCast::from(abstract_self);
        let (chan, port) = channel();
        let addr = node.to_trusted_node_address();
        let ContentBoxResponse(rect) = win.get().page().query_layout(ContentBoxQuery(addr, chan), port);
        ClientRect::new(
            win,
            rect.origin.y,
            rect.origin.y + rect.size.height,
            rect.origin.x,
            rect.origin.x + rect.size.width)
    }

    pub fn GetInnerHTML(&self, abstract_self: &JS<Element>) -> Fallible<DOMString> {
        //XXX TODO: XML case
        Ok(serialize(&mut NodeIterator::new(NodeCast::from(abstract_self), false, false)))
    }

    pub fn GetOuterHTML(&self, abstract_self: &JS<Element>) -> Fallible<DOMString> {
        Ok(serialize(&mut NodeIterator::new(NodeCast::from(abstract_self), true, false)))
    }
}

pub trait IElement {
    fn bind_to_tree_impl(&self);
    fn unbind_from_tree_impl(&self);
}

impl IElement for JS<Element> {
    fn bind_to_tree_impl(&self) {
        match self.get_attribute(Null, "id") {
            Some(attr) => {
                let mut doc = document_from_node(self);
                doc.get_mut().register_named_element(self, attr.get().Value());
            }
            _ => ()
        }
    }

    fn unbind_from_tree_impl(&self) {
        match self.get_attribute(Null, "id") {
            Some(attr) => {
                let mut doc = document_from_node(self);
                doc.get_mut().unregister_named_element(self, attr.get().Value());
            }
            _ => ()
        }
    }
}

pub fn get_attribute_parts(name: DOMString) -> (Option<~str>, ~str) {
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
