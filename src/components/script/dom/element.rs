/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::{Attr, ReplacedAttr, FirstSetAttr, AttrMethods};
use dom::attrlist::AttrList;
use dom::bindings::codegen::BindingDeclarations::ElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementDerived, NodeCast};
use dom::bindings::js::{JS, JSRef, RootCollection, Unrooted, UnrootedPushable};
use dom::bindings::js::{OptionalAssignable, OptionalRootable, Root};
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
use dom::node::window_from_node;
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
    pub node: Node,
    pub local_name: DOMString,     // TODO: This should be an atom, not a DOMString.
    pub namespace: Namespace,
    pub prefix: Option<DOMString>,
    pub attrs: Vec<JS<Attr>>,
    pub style_attribute: Option<style::PropertyDeclarationBlock>,
    pub attr_list: Option<JS<AttrList>>
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
            attrs: vec!(),
            attr_list: None,
            style_attribute: None,
        }
    }

    pub fn new(local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: &JSRef<Document>) -> Unrooted<Element> {
        let element = Element::new_inherited(ElementTypeId, local_name, namespace, prefix, document.unrooted());
        Node::reflect_node(~element, document, ElementBinding::Wrap)
    }

    pub fn html_element_in_html_document(&self) -> bool {
        let roots = RootCollection::new();
        self.namespace == namespace::HTML &&
        self.node.owner_doc().root(&roots).is_html_document
    }
}

impl Element {
    pub unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if self.namespace != namespace::HTML {
            return false
        }
        let owner_doc: *JS<Document> = self.node.owner_doc_for_layout();
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
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<Unrooted<Attr>>;
    fn set_attr(&mut self, name: DOMString, value: DOMString) -> ErrorResult;
    fn set_attribute(&mut self, namespace: Namespace, name: DOMString,
                     value: DOMString) -> ErrorResult;
    fn do_set_attribute(&mut self, local_name: DOMString, value: DOMString,
                        name: DOMString, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |&JSRef<Attr>| -> bool);
    fn SetAttribute_(&mut self, name: DOMString, value: DOMString) -> ErrorResult;
    fn SetAttributeNS_(&mut self, namespace_url: Option<DOMString>,
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

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<Unrooted<Attr>> {
        let roots = RootCollection::new();
        if self.get().html_element_in_html_document() {
            self.get().attrs.iter().map(|attr| attr.root(&roots)).find(|attr| {
                name.to_ascii_lower() == attr.local_name && attr.namespace == namespace
            }).map(|x| Unrooted::new_rooted(&*x))
        } else {
            self.get().attrs.iter().map(|attr| attr.root(&roots)).find(|attr| {
                name == attr.local_name && attr.namespace == namespace
            }).map(|x| Unrooted::new_rooted(&*x))
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

        let self_alias = self.clone();
        let node: &JSRef<Node> = NodeCast::from_ref(&self_alias);
        node.wait_until_safe_to_modify_dom();

        let position: |&JSRef<Attr>| -> bool =
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
                        prefix: Option<DOMString>, cb: |&JSRef<Attr>| -> bool) {
        let roots = RootCollection::new();
        let idx = self.get().attrs.iter()
                                  .map(|attr| attr.root(&roots))
                                  .position(|attr| cb(&*attr));
        let (idx, set_type) = match idx {
            Some(idx) => (idx, ReplacedAttr),
            None => {
                let window = window_from_node(self).root(&roots);
                let attr = Attr::new(&*window, local_name.clone(), value.clone(),
                                     name, namespace.clone(), prefix, self);
                self.get_mut().attrs.push_unrooted(attr);
                (self.get().attrs.len() - 1, FirstSetAttr)
            }
        };

        self.get_mut().attrs.get(idx).root(&roots).set_value(set_type, value);
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute_(&mut self, name: DOMString, value: DOMString) -> ErrorResult {
        {
            let node: &JSRef<Node> = NodeCast::from_ref(self);
            node.get().wait_until_safe_to_modify_dom();
        }

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

    fn SetAttributeNS_(&mut self, namespace_url: Option<DOMString>,
                       name: DOMString, value: DOMString) -> ErrorResult {
        {
            let node: &JSRef<Node> = NodeCast::from_ref(self);
            node.get().wait_until_safe_to_modify_dom();
        }

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
        let roots = RootCollection::new();
        let (_, local_name) = get_attribute_parts(name.clone());

        let self_alias = self.clone();
        let node: &JSRef<Node> = NodeCast::from_ref(&self_alias);
        node.wait_until_safe_to_modify_dom();

        let idx = self.get().attrs.iter().map(|attr| attr.root(&roots)).position(|attr| {
            attr.local_name == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                if namespace == namespace::Null {
                    let removed_raw_value = self.get().attrs.get(idx).root(&roots).Value();
                    vtable_for(node).before_remove_attr(local_name.clone(), removed_raw_value);
                }

                self.get_mut().attrs.remove(idx);
            }
        };

        Ok(())
    }

    fn notify_attribute_changed(&self, local_name: DOMString) {
        let roots = RootCollection::new();
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        if node.is_in_doc() {
            let damage = match local_name.as_slice() {
                "style" | "id" | "class" => MatchSelectorsDocumentDamage,
                _ => ContentChangedDocumentDamage
            };
            let document = node.owner_doc().root(&roots);
            document.deref().damage_and_reflow(damage);
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
        let roots = RootCollection::new();
        match self.get_attribute(Null, name) {
            Some(x) => {
                let x = x.root(&roots);
                x.deref().Value()
            }
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

pub trait ElementMethods {
    fn NamespaceURI(&self) -> DOMString;
    fn LocalName(&self) -> DOMString;
    fn GetPrefix(&self) -> Option<DOMString>;
    fn TagName(&self) -> DOMString;
    fn Id(&self, abstract_self: &JSRef<Element>) -> DOMString;
    fn SetId(&mut self, abstract_self: &mut JSRef<Element>, id: DOMString);
    fn ClassName(&self, abstract_self: &JSRef<Element>) -> DOMString;
    fn SetClassName(&self, abstract_self: &mut JSRef<Element>, class: DOMString);
    fn Attributes(&mut self, abstract_self: &JSRef<Element>) -> Unrooted<AttrList>;
    fn GetAttribute(&self, abstract_self: &JSRef<Element>, name: DOMString) -> Option<DOMString>;
    fn GetAttributeNS(&self, abstract_self: &JSRef<Element>, namespace: Option<DOMString>, local_name: DOMString) -> Option<DOMString>;
    fn SetAttribute(&self, abstract_self: &mut JSRef<Element>, name: DOMString, value: DOMString) -> ErrorResult;
    fn SetAttributeNS(&self, abstract_self: &mut JSRef<Element>, namespace_url: Option<DOMString>, name: DOMString, value: DOMString) -> ErrorResult;
    fn RemoveAttribute(&mut self, abstract_self: &mut JSRef<Element>, name: DOMString) -> ErrorResult;
    fn RemoveAttributeNS(&mut self, abstract_self: &mut JSRef<Element>, namespace: Option<DOMString>, localname: DOMString) -> ErrorResult;
    fn HasAttribute(&self, abstract_self: &JSRef<Element>, name: DOMString) -> bool;
    fn HasAttributeNS(&self, abstract_self: &JSRef<Element>, namespace: Option<DOMString>, local_name: DOMString) -> bool;
    fn GetElementsByTagName(&self, abstract_self: &JSRef<Element>, localname: DOMString) -> Unrooted<HTMLCollection>;
    fn GetElementsByTagNameNS(&self, abstract_self: &JSRef<Element>, maybe_ns: Option<DOMString>, localname: DOMString) -> Unrooted<HTMLCollection>;
    fn GetElementsByClassName(&self, abstract_self: &JSRef<Element>, classes: DOMString) -> Unrooted<HTMLCollection>;
    fn GetClientRects(&self, abstract_self: &JSRef<Element>) -> Unrooted<ClientRectList>;
    fn GetBoundingClientRect(&self, abstract_self: &JSRef<Element>) -> Unrooted<ClientRect>;
    fn GetInnerHTML(&self, abstract_self: &JSRef<Element>) -> Fallible<DOMString>;
    fn GetOuterHTML(&self, abstract_self: &JSRef<Element>) -> Fallible<DOMString>;
    fn Children(&self, abstract_self: &JSRef<Element>) -> Unrooted<HTMLCollection>;
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn NamespaceURI(&self) -> DOMString {
        self.namespace.to_str().to_owned()
    }

    fn LocalName(&self) -> DOMString {
        self.local_name.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(&self) -> DOMString {
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
    fn Id(&self, abstract_self: &JSRef<Element>) -> DOMString {
        abstract_self.get_string_attribute("id")
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn SetId(&mut self, abstract_self: &mut JSRef<Element>, id: DOMString) {
        abstract_self.set_string_attribute("id", id);
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(&self, abstract_self: &JSRef<Element>) -> DOMString {
        abstract_self.get_string_attribute("class")
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(&self, abstract_self: &mut JSRef<Element>, class: DOMString) {
        abstract_self.set_string_attribute("class", class);
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(&mut self, abstract_self: &JSRef<Element>) -> Unrooted<AttrList> {
        let roots = RootCollection::new();
        match self.attr_list {
            None => (),
            Some(ref list) => return Unrooted::new(list.clone()),
        }

        let doc = self.node.owner_doc().root(&roots);
        let window = doc.deref().window.root(&roots);
        let list = AttrList::new(&window.root_ref(), abstract_self);
        self.attr_list.assign(Some(list));
        Unrooted::new(self.attr_list.get_ref().clone())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(&self, abstract_self: &JSRef<Element>, name: DOMString) -> Option<DOMString> {
        let roots = RootCollection::new();
        let name = if abstract_self.get().html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        abstract_self.get_attribute(Null, name).root(&roots)
                     .map(|s| s.deref().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(&self, abstract_self: &JSRef<Element>,
                          namespace: Option<DOMString>,
                          local_name: DOMString) -> Option<DOMString> {
        let roots = RootCollection::new();
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        abstract_self.get_attribute(namespace, local_name).root(&roots)
                     .map(|attr| attr.deref().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(&self, abstract_self: &mut JSRef<Element>,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        abstract_self.SetAttribute_(name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(&self,
                      abstract_self: &mut JSRef<Element>,
                      namespace_url: Option<DOMString>,
                      name: DOMString,
                      value: DOMString) -> ErrorResult {
        abstract_self.SetAttributeNS_(namespace_url, name, value)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(&mut self,
                       abstract_self: &mut JSRef<Element>,
                       name: DOMString) -> ErrorResult {
        let name = if self.html_element_in_html_document() {
            name.to_ascii_lower()
        } else {
            name
        };
        abstract_self.remove_attribute(namespace::Null, name)
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(&mut self,
                             abstract_self: &mut JSRef<Element>,
                             namespace: Option<DOMString>,
                             localname: DOMString) -> ErrorResult {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        abstract_self.remove_attribute(namespace, localname)
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(&self, abstract_self: &JSRef<Element>,
                        name: DOMString) -> bool {
        self.GetAttribute(abstract_self, name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(&self, abstract_self: &JSRef<Element>,
                          namespace: Option<DOMString>,
                          local_name: DOMString) -> bool {
        self.GetAttributeNS(abstract_self, namespace, local_name).is_some()
    }

    fn GetElementsByTagName(&self, abstract_self: &JSRef<Element>, localname: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = window_from_node(abstract_self).root(&roots);
        HTMLCollection::by_tag_name(&*window, NodeCast::from_ref(abstract_self), localname)
    }

    fn GetElementsByTagNameNS(&self, abstract_self: &JSRef<Element>, maybe_ns: Option<DOMString>,
                                  localname: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace),
            None => Null
        };
        let window = window_from_node(abstract_self).root(&roots);
        HTMLCollection::by_tag_name_ns(&*window, NodeCast::from_ref(abstract_self), localname, namespace)
    }

    fn GetElementsByClassName(&self, abstract_self: &JSRef<Element>, classes: DOMString) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = window_from_node(abstract_self).root(&roots);
        HTMLCollection::by_class_name(&*window, NodeCast::from_ref(abstract_self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self, abstract_self: &JSRef<Element>) -> Unrooted<ClientRectList> {
        let roots = RootCollection::new();
        let win = window_from_node(abstract_self).root(&roots);
        let node: &JSRef<Node> = NodeCast::from_ref(abstract_self);
        let rects = node.get_content_boxes();
        let rects: ~[Root<ClientRect>] = rects.iter().map(|r| {
            ClientRect::new(
                &*win,
                r.origin.y,
                r.origin.y + r.size.height,
                r.origin.x,
                r.origin.x + r.size.width).root(&roots)
        }).collect();

        ClientRectList::new(&*win, rects.iter().map(|rect| rect.deref().clone()).collect())
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self, abstract_self: &JSRef<Element>) -> Unrooted<ClientRect> {
        let roots = RootCollection::new();
        let win = window_from_node(abstract_self).root(&roots);
        let node: &JSRef<Node> = NodeCast::from_ref(abstract_self);
        let rect = node.get_bounding_content_box();
        ClientRect::new(
            &*win,
            rect.origin.y,
            rect.origin.y + rect.size.height,
            rect.origin.x,
            rect.origin.x + rect.size.width)
    }

    fn GetInnerHTML(&self, abstract_self: &JSRef<Element>) -> Fallible<DOMString> {
        //XXX TODO: XML case
        let roots = RootCollection::new();
        Ok(serialize(&mut NodeIterator::new(&roots, NodeCast::from_ref(abstract_self), false, false)))
    }

    fn GetOuterHTML(&self, abstract_self: &JSRef<Element>) -> Fallible<DOMString> {
        let roots = RootCollection::new();
        Ok(serialize(&mut NodeIterator::new(&roots, NodeCast::from_ref(abstract_self), true, false)))
    }

    fn Children(&self, abstract_self: &JSRef<Element>) -> Unrooted<HTMLCollection> {
        let roots = RootCollection::new();
        let window = window_from_node(abstract_self).root(&roots);
        HTMLCollection::children(&window.root_ref(), NodeCast::from_ref(abstract_self))
    }
}

pub fn get_attribute_parts(name: DOMString) -> (Option<~str>, ~str) {
    //FIXME: Throw for XML-invalid names
    //FIXME: Throw for XMLNS-invalid names
    let (prefix, local_name) = if name.contains(":")  {
        let mut parts = name.splitn(':', 1);
        (Some(parts.next().unwrap().to_owned()), parts.next().unwrap().to_owned())
    } else {
        (None, name)
    };

    (prefix, local_name)
}

impl<'a> VirtualMethods for JSRef<'a, Element> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        Some(~node.clone() as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        let roots = RootCollection::new();
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                let doc = document_from_node(self).root(&roots);
                let base_url = doc.get().url().clone();
                self.get_mut().style_attribute = Some(style::parse_style_attribute(value, &base_url))
            }
            "id" => {
                let node: &JSRef<Node> = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    let mut doc = document_from_node(self).root(&roots);
                    let doc_alias = (*doc).clone();
                    doc.register_named_element(&doc_alias, self, value.clone());
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn before_remove_attr(&mut self, name: DOMString, value: DOMString) {
        let roots = RootCollection::new();
        match self.super_type() {
            Some(ref mut s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                self.get_mut().style_attribute = None
            }
            "id" => {
                let node: &JSRef<Node> = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    let mut doc = document_from_node(self).root(&roots);
                    doc.unregister_named_element(self, value);
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn bind_to_tree(&mut self) {
        let roots = RootCollection::new();
        match self.super_type() {
            Some(ref mut s) => s.bind_to_tree(),
            _ => (),
        }

        match self.get_attribute(Null, "id").root(&roots) {
            Some(attr) => {
                let mut doc = document_from_node(self).root(&roots);
                let doc_alias = (*doc).clone();
                doc.register_named_element(&doc_alias, self, attr.deref().Value());
            }
            _ => ()
        }
    }

    fn unbind_from_tree(&mut self) {
        let roots = RootCollection::new();
        match self.super_type() {
            Some(ref mut s) => s.unbind_from_tree(),
            _ => (),
        }

        match self.get_attribute(Null, "id").root(&roots) {
            Some(attr) => {
                let mut doc = document_from_node(self).root(&roots);
                doc.unregister_named_element(self, attr.deref().Value());
            }
            _ => ()
        }
    }
}
