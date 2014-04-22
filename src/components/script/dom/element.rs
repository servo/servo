/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::{Attr, AttrSettingType, ReplacedAttr, FirstSetAttr};
use dom::attrlist::AttrList;
use dom::bindings::codegen::ElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementDerived, NodeCast};
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{ErrorResult, Fallible, NamespaceError, InvalidCharacter};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::Document;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlserializer::serialize;
use dom::node::{ElementNodeTypeId, Node, NodeHelpers, NodeIterator, document_from_node};
use dom::virtualmethods::{VirtualMethods, vtable_for};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface::MatchSelectorsDocumentDamage;
use style;
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref, split_html_space_chars};

use std::ascii::StrAsciiExt;
use std::cast;

#[deriving(Encodable)]
pub struct Element {
    node: Node,
    local_name: DOMString,     // TODO: This should be an atom, not a DOMString.
    namespace: Namespace,
    prefix: Option<DOMString>,
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
    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: JS<Document>) -> Element {
        Element {
            node: Node::new_inherited(ElementNodeTypeId(type_id), document),
            local_name: local_name,
            namespace: namespace,
            prefix: prefix,
            attrs: ~[],
            attr_list: None,
            style_attribute: None,
        }
    }

    pub fn new(local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: &JS<Document>) -> JS<Element> {
        let element = Element::new_inherited(ElementTypeId, local_name, namespace, prefix, document.clone());
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
    fn do_set_attribute(&mut self, local_name: DOMString, value: DOMString,
                        name: DOMString, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |&JS<Attr>| -> bool);
    fn SetAttribute(&mut self, name: DOMString, value: DOMString) -> ErrorResult;
    fn SetAttributeNS(&mut self, namespace_url: Option<DOMString>,
                      name: DOMString, value: DOMString) -> ErrorResult;

    fn remove_attribute(&mut self, namespace: Namespace, name: DOMString) -> ErrorResult;
    fn notify_attribute_changed(&self, local_name: DOMString);
    fn has_class(&self, name: &str) -> bool;

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn get_url_attribute(&self, name: &str) -> DOMString;
    fn set_url_attribute(&mut self, name: &str, value: DOMString);
    fn get_string_attribute(&self, name: &str) -> DOMString;
    fn set_string_attribute(&mut self, name: &str, value: DOMString);
    fn set_uint_attribute(&mut self, name: &str, value: u32);
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

        let position: |&JS<Attr>| -> bool =
            if self.get().html_element_in_html_document() {
                |attr| attr.get().local_name.eq_ignore_ascii_case(local_name)
            } else {
                |attr| attr.get().local_name == local_name
            };
        self.do_set_attribute(name.clone(), value, name.clone(), namespace::Null, None, position);
        Ok(())
    }

    fn do_set_attribute(&mut self, local_name: DOMString, value: DOMString,
                        name: DOMString, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |&JS<Attr>| -> bool) {
        let node: JS<Node> = NodeCast::from(self);
        let idx = self.get().attrs.iter().position(cb);
        let (mut attr, set_type): (JS<Attr>, AttrSettingType) = match idx {
            Some(idx) => {
                let attr = self.get_mut().attrs[idx].clone();
                (attr, ReplacedAttr)
            }

            None => {
                let doc = node.get().owner_doc().get();
                let attr = Attr::new(&doc.window, local_name.clone(), value.clone(),
                                         name, namespace.clone(), prefix, self.clone());
                self.get_mut().attrs.push(attr.clone());
                (attr, FirstSetAttr)
            }
        };

        attr.get_mut().set_value(set_type, value);
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(&mut self, name: DOMString, value: DOMString) -> ErrorResult {
        let node: JS<Node> = NodeCast::from(self);
        node.get().wait_until_safe_to_modify_dom();

        // Step 1.
        match xml_name_type(name) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Step 2.
        let name = if self.get().html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };

        // Step 3-5.
        self.do_set_attribute(name.clone(), value, name.clone(), namespace::Null, None, |attr| {
            attr.get().name == name
        });
        Ok(())
    }

    fn SetAttributeNS(&mut self, namespace_url: Option<DOMString>,
                      name: DOMString, value: DOMString) -> ErrorResult {
        let node: JS<Node> = NodeCast::from(self);
        node.get().wait_until_safe_to_modify_dom();

        // Step 1.
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace_url));

        let name_type = xml_name_type(name);
        match name_type {
            // Step 2.
            InvalidXMLName => return Err(InvalidCharacter),
            // Step 3.
            Name => return Err(NamespaceError),
            QName => {}
        }

        // Step 4.
        let (prefix, local_name) = get_attribute_parts(name.clone());
        match prefix {
            Some(ref prefix_str) => {
                // Step 5.
                if namespace == namespace::Null {
                    return Err(NamespaceError);
                }

                // Step 6.
                if "xml" == prefix_str.as_slice() && namespace != namespace::XML {
                    return Err(NamespaceError);
                }

                // Step 7b.
                if "xmlns" == prefix_str.as_slice() && namespace != namespace::XMLNS {
                    return Err(NamespaceError);
                }
            },
            None => {}
        }

        // Step 7a.
        if "xmlns" == name && namespace != namespace::XMLNS {
            return Err(NamespaceError);
        }

        // Step 8.
        if namespace == namespace::XMLNS && "xmlns" != name && Some(~"xmlns") != prefix {
            return Err(NamespaceError);
        }

        // Step 9.
        self.do_set_attribute(local_name.clone(), value, name, namespace.clone(), prefix, |attr| {
            attr.get().local_name == local_name &&
            attr.get().namespace == namespace
        });
        Ok(())
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
                    vtable_for(&node).before_remove_attr(local_name.clone(), removed_raw_value);
                }

                self.get_mut().attrs.remove(idx);
            }
        };

        Ok(())
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
        match self.local_name.as_slice() {
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

    pub fn LocalName(&self) -> DOMString {
        self.local_name.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-prefix
    pub fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    pub fn TagName(&self) -> DOMString {
        match self.prefix {
            None => {
                self.local_name.to_ascii_upper()
            }
            Some(ref prefix_str) => {
                (*prefix_str + ":" + self.local_name).to_ascii_upper()
            }
        }
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
    pub fn SetAttribute(&self, abstract_self: &mut JS<Element>,
                        name: DOMString,
                        value: DOMString) -> ErrorResult {
        abstract_self.SetAttribute(name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    pub fn SetAttributeNS(&self,
                          abstract_self: &mut JS<Element>,
                          namespace_url: Option<DOMString>,
                          name: DOMString,
                          value: DOMString) -> ErrorResult {
        abstract_self.SetAttributeNS(namespace_url, name, value)
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
        let rects = node.get_content_boxes();
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
        let rect = node.get_bounding_content_box();
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

    pub fn Children(&self, abstract_self: &JS<Element>) -> JS<HTMLCollection> {
        let doc = self.node.owner_doc();
        let doc = doc.get();
        HTMLCollection::children(&doc.window, &NodeCast::from(abstract_self))
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

impl VirtualMethods for JS<Element> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let node: JS<Node> = NodeCast::from(self);
        Some(~node as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        let node: JS<Node> = NodeCast::from(self);
        match name.as_slice() {
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

        self.notify_attribute_changed(name);
    }

    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        let node: JS<Node> = NodeCast::from(self);
        match name.as_slice() {
            "style" => {
                self.get_mut().style_attribute = None
            }
            "id" if node.is_in_doc() => {
                // XXX: this dual declaration are workaround to avoid the compile error:
                // "borrowed value does not live long enough"
                let mut doc = node.get().owner_doc().clone();
                let doc = doc.get_mut();
                doc.unregister_named_element(self, value);
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn bind_to_tree(&mut self) {
        match self.super_type() {
            Some(ref mut s) => s.bind_to_tree(),
            _ => (),
        }

        match self.get_attribute(Null, "id") {
            Some(attr) => {
                let mut doc = document_from_node(self);
                doc.get_mut().register_named_element(self, attr.get().Value());
            }
            _ => ()
        }
    }

    fn unbind_from_tree(&mut self) {
        match self.super_type() {
            Some(ref mut s) => s.unbind_from_tree(),
            _ => (),
        }

        match self.get_attribute(Null, "id") {
            Some(attr) => {
                let mut doc = document_from_node(self);
                doc.get_mut().unregister_named_element(self, attr.get().Value());
            }
            _ => ()
        }
    }
}
