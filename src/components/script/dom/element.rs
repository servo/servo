/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::{Attr, ReplacedAttr, FirstSetAttr, AttrMethods, AttrHelpersForLayout};
use dom::attr::{AttrValue, StringAttrValue, UIntAttrValue};
use dom::attrlist::AttrList;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementDerived, NodeCast};
use dom::bindings::js::{JS, JSRef, Temporary, TemporaryPushable};
use dom::bindings::js::{OptionalSettable, OptionalRootable, Root};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{ErrorResult, Fallible, NamespaceError, InvalidCharacter};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::clientrect::ClientRect;
use dom::clientrectlist::ClientRectList;
use dom::document::{Document, DocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlserializer::serialize;
use dom::node::{ElementNodeTypeId, Node, NodeHelpers, NodeIterator, document_from_node};
use dom::node::{window_from_node, LayoutNodeHelpers};
use dom::nodelist::NodeList;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use layout_interface::ContentChangedDocumentDamage;
use layout_interface::MatchSelectorsDocumentDamage;
use style;
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref, split_html_space_chars};
use servo_util::atom::Atom;

use std::ascii::StrAsciiExt;
use std::cell::{Cell, RefCell};
use std::mem;

#[deriving(Encodable)]
pub struct Element {
    pub node: Node,
    pub local_name: Atom,
    pub namespace: Namespace,
    pub prefix: Option<DOMString>,
    pub attrs: RefCell<Vec<JS<Attr>>>,
    pub style_attribute: Traceable<RefCell<Option<style::PropertyDeclarationBlock>>>,
    pub attr_list: Cell<Option<JS<AttrList>>>,
    class_list: Cell<Option<JS<DOMTokenList>>>,
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
}

#[deriving(PartialEq,Encodable)]
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
    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: &JSRef<Document>) -> Element {
        Element {
            node: Node::new_inherited(ElementNodeTypeId(type_id), document),
            local_name: Atom::from_slice(local_name.as_slice()),
            namespace: namespace,
            prefix: prefix,
            attrs: RefCell::new(vec!()),
            attr_list: Cell::new(None),
            class_list: Cell::new(None),
            style_attribute: Traceable::new(RefCell::new(None)),
        }
    }

    pub fn new(local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: &JSRef<Document>) -> Temporary<Element> {
        let element = Element::new_inherited(ElementTypeId, local_name, namespace, prefix, document);
        Node::reflect_node(box element, document, ElementBinding::Wrap)
    }
}

pub trait RawLayoutElementHelpers {
    unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str) -> Option<&'static str>;
}

impl RawLayoutElementHelpers for Element {
    #[inline]
    unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str)
                                      -> Option<&'static str> {
        // cast to point to T in RefCell<T> directly
        let attrs: *Vec<JS<Attr>> = mem::transmute(&self.attrs);
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            name == (*attr).local_name.as_slice() && (*attr).namespace == *namespace
        }).map(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_ref_forever()
        })
    }
}

pub trait LayoutElementHelpers {
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
}

impl LayoutElementHelpers for JS<Element> {
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if (*self.unsafe_get()).namespace != namespace::HTML {
            return false
        }
        let node: JS<Node> = self.transmute_copy();
        let owner_doc = node.owner_doc_for_layout().unsafe_get();
        (*owner_doc).is_html_document
    }
}

pub trait ElementHelpers {
    fn html_element_in_html_document(&self) -> bool;
    fn get_local_name<'a>(&'a self) -> &'a str;
    fn get_namespace<'a>(&'a self) -> &'a Namespace;
}

impl<'a> ElementHelpers for JSRef<'a, Element> {
    fn html_element_in_html_document(&self) -> bool {
        let is_html = self.namespace == namespace::HTML;
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        is_html && node.owner_doc().root().is_html_document
    }

    fn get_local_name<'a>(&'a self) -> &'a str {
        self.deref().local_name.as_slice()
    }

    fn get_namespace<'a>(&'a self) -> &'a Namespace {
        &self.deref().namespace
    }
}

pub trait AttributeHandlers {
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<Temporary<Attr>>;
    fn set_attribute_from_parser(&self, local_name: DOMString,
                                 value: DOMString, namespace: Namespace,
                                 prefix: Option<DOMString>);
    fn set_attribute(&self, name: &str, value: AttrValue);
    fn do_set_attribute(&self, local_name: DOMString, value: AttrValue,
                        name: DOMString, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |&JSRef<Attr>| -> bool);
    fn parse_attribute(&self, namespace: &Namespace, local_name: &str,
                       value: DOMString) -> AttrValue;

    fn remove_attribute(&self, namespace: Namespace, name: &str) -> ErrorResult;
    fn notify_attribute_changed(&self, local_name: DOMString);
    fn has_class(&self, name: &str) -> bool;

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn get_url_attribute(&self, name: &str) -> DOMString;
    fn set_url_attribute(&self, name: &str, value: DOMString);
    fn get_string_attribute(&self, name: &str) -> DOMString;
    fn set_string_attribute(&self, name: &str, value: DOMString);
    fn set_tokenlist_attribute(&self, name: &str, value: DOMString);
    fn get_uint_attribute(&self, name: &str) -> u32;
    fn set_uint_attribute(&self, name: &str, value: u32);
}

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(&self, namespace: Namespace, name: &str) -> Option<Temporary<Attr>> {
        let element: &Element = self.deref();
        let is_html_element = self.html_element_in_html_document();

        element.attrs.borrow().iter().map(|attr| attr.root()).find(|attr| {
            let same_name = if is_html_element {
                name.to_ascii_lower() == attr.local_name
            } else {
                name == attr.local_name.as_slice()
            };

            same_name && attr.namespace == namespace
        }).map(|x| Temporary::from_rooted(&*x))
    }

    fn set_attribute_from_parser(&self, local_name: DOMString,
                                 value: DOMString, namespace: Namespace,
                                 prefix: Option<DOMString>) {
        let name = match prefix {
            None => local_name.clone(),
            Some(ref prefix) => format!("{:s}:{:s}", *prefix, local_name),
        };
        let value = self.parse_attribute(&namespace, local_name.as_slice(), value);
        self.do_set_attribute(local_name, value, name, namespace, prefix, |_| false)
    }

    fn set_attribute(&self, name: &str, value: AttrValue) {
        assert!(name == name.to_ascii_lower().as_slice());
        assert!(!name.contains(":"));

        let node: &JSRef<Node> = NodeCast::from_ref(self);
        node.wait_until_safe_to_modify_dom();

        self.do_set_attribute(name.to_string(), value, name.to_string(),
            namespace::Null, None,
            |attr| attr.deref().local_name.as_slice() == name);
    }

    fn do_set_attribute(&self, local_name: DOMString, value: AttrValue,
                        name: DOMString, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |&JSRef<Attr>| -> bool) {
        let idx = self.deref().attrs.borrow().iter()
                                    .map(|attr| attr.root())
                                    .position(|attr| cb(&*attr));
        let (idx, set_type) = match idx {
            Some(idx) => (idx, ReplacedAttr),
            None => {
                let window = window_from_node(self).root();
                let attr = Attr::new(&*window, local_name.clone(), value.clone(),
                                     name, namespace.clone(), prefix, self);
                self.deref().attrs.borrow_mut().push_unrooted(&attr);
                (self.deref().attrs.borrow().len() - 1, FirstSetAttr)
            }
        };

        self.deref().attrs.borrow().get(idx).root().set_value(set_type, value);
    }

    fn parse_attribute(&self, namespace: &Namespace, local_name: &str,
                       value: DOMString) -> AttrValue {
        if *namespace == namespace::Null {
            vtable_for(NodeCast::from_ref(self))
                .parse_plain_attribute(local_name, value)
        } else {
            StringAttrValue(value)
        }
    }

    fn remove_attribute(&self, namespace: Namespace, name: &str) -> ErrorResult {
        let (_, local_name) = get_attribute_parts(name);

        let idx = self.deref().attrs.borrow().iter().map(|attr| attr.root()).position(|attr| {
            attr.local_name.as_slice() == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                {
                    let node: &JSRef<Node> = NodeCast::from_ref(self);
                    node.wait_until_safe_to_modify_dom();
                }

                if namespace == namespace::Null {
                    let removed_raw_value = self.deref().attrs.borrow().get(idx).root().Value();
                    vtable_for(NodeCast::from_ref(self))
                        .before_remove_attr(local_name.to_string(), removed_raw_value);
                }

                self.deref().attrs.borrow_mut().remove(idx);
            }
        };

        Ok(())
    }

    fn notify_attribute_changed(&self, local_name: DOMString) {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        if node.is_in_doc() {
            let damage = match local_name.as_slice() {
                "style" | "id" | "class" => MatchSelectorsDocumentDamage,
                _ => ContentChangedDocumentDamage
            };
            let document = node.owner_doc().root();
            document.deref().damage_and_reflow(damage);
        }
    }

    fn has_class(&self, name: &str) -> bool {
        let class_names = self.get_string_attribute("class");
        let mut classes = split_html_space_chars(class_names.as_slice());
        classes.any(|class| name == class)
    }

    fn get_url_attribute(&self, name: &str) -> DOMString {
        // XXX Resolve URL.
        self.get_string_attribute(name)
    }
    fn set_url_attribute(&self, name: &str, value: DOMString) {
        self.set_string_attribute(name, value);
    }

    fn get_string_attribute(&self, name: &str) -> DOMString {
        match self.get_attribute(Null, name) {
            Some(x) => {
                let x = x.root();
                x.deref().Value()
            }
            None => "".to_string()
        }
    }
    fn set_string_attribute(&self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, StringAttrValue(value));
    }

    fn set_tokenlist_attribute(&self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, AttrValue::from_tokenlist(value));
    }

    fn get_uint_attribute(&self, name: &str) -> u32 {
        assert!(name == name.to_ascii_lower().as_slice());
        let attribute = self.get_attribute(Null, name).root();
        match attribute {
            Some(attribute) => {
                match *attribute.deref().value() {
                    UIntAttrValue(_, value) => value,
                    _ => fail!("Expected a UIntAttrValue"),
                }
            }
            None => 0,
        }
    }
    fn set_uint_attribute(&self, name: &str, value: u32) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, UIntAttrValue(value.to_str(), value));
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
    fn Id(&self) -> DOMString;
    fn SetId(&self, id: DOMString);
    fn ClassName(&self) -> DOMString;
    fn SetClassName(&self, class: DOMString);
    fn ClassList(&self) -> Temporary<DOMTokenList>;
    fn Attributes(&self) -> Temporary<AttrList>;
    fn GetAttribute(&self, name: DOMString) -> Option<DOMString>;
    fn GetAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> Option<DOMString>;
    fn SetAttribute(&self, name: DOMString, value: DOMString) -> ErrorResult;
    fn SetAttributeNS(&self, namespace_url: Option<DOMString>, name: DOMString, value: DOMString) -> ErrorResult;
    fn RemoveAttribute(&self, name: DOMString) -> ErrorResult;
    fn RemoveAttributeNS(&self, namespace: Option<DOMString>, localname: DOMString) -> ErrorResult;
    fn HasAttribute(&self, name: DOMString) -> bool;
    fn HasAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> bool;
    fn GetElementsByTagName(&self, localname: DOMString) -> Temporary<HTMLCollection>;
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>, localname: DOMString) -> Temporary<HTMLCollection>;
    fn GetElementsByClassName(&self, classes: DOMString) -> Temporary<HTMLCollection>;
    fn GetClientRects(&self) -> Temporary<ClientRectList>;
    fn GetBoundingClientRect(&self) -> Temporary<ClientRect>;
    fn GetInnerHTML(&self) -> Fallible<DOMString>;
    fn GetOuterHTML(&self) -> Fallible<DOMString>;
    fn Children(&self) -> Temporary<HTMLCollection>;
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>>;
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Temporary<NodeList>>;
    fn Remove(&self);
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn NamespaceURI(&self) -> DOMString {
        self.namespace.to_str().to_string()
    }

    fn LocalName(&self) -> DOMString {
        self.local_name.as_slice().to_string()
    }

    // http://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(&self) -> DOMString {
        match self.prefix {
            None => {
                self.local_name.as_slice().to_ascii_upper()
            }
            Some(ref prefix_str) => {
                let s = format!("{}:{}", prefix_str, self.local_name);
                s.as_slice().to_ascii_upper()
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn Id(&self) -> DOMString {
        self.get_string_attribute("id")
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn SetId(&self, id: DOMString) {
        self.set_string_attribute("id", id);
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(&self) -> DOMString {
        self.get_string_attribute("class")
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(&self, class: DOMString) {
        self.set_tokenlist_attribute("class", class);
    }

    // http://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(&self) -> Temporary<DOMTokenList> {
        match self.class_list.get() {
            Some(class_list) => Temporary::new(class_list),
            None => {
                let class_list = DOMTokenList::new(self, "class").root();
                self.class_list.assign(Some(class_list.deref().clone()));
                Temporary::from_rooted(&*class_list)
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(&self) -> Temporary<AttrList> {
        match self.attr_list.get() {
            None => (),
            Some(ref list) => return Temporary::new(list.clone()),
        }

        let doc = {
            let node: &JSRef<Node> = NodeCast::from_ref(self);
            node.owner_doc().root()
        };
        let window = doc.deref().window.root();
        let list = AttrList::new(&*window, self);
        self.attr_list.assign(Some(list));
        Temporary::new(self.attr_list.get().get_ref().clone())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };
        self.get_attribute(Null, name.as_slice()).root()
                     .map(|s| s.deref().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(&self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.get_attribute(namespace, local_name.as_slice()).root()
                     .map(|attr| attr.deref().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(&self,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        {
            let node: &JSRef<Node> = NodeCast::from_ref(self);
            node.wait_until_safe_to_modify_dom();
        }

        // Step 1.
        match xml_name_type(name.as_slice()) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Step 2.
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };

        // Step 3-5.
        let value = self.parse_attribute(&namespace::Null, name.as_slice(), value);
        self.do_set_attribute(name.clone(), value, name.clone(), namespace::Null, None, |attr| {
            attr.deref().name == name
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(&self,
                      namespace_url: Option<DOMString>,
                      name: DOMString,
                      value: DOMString) -> ErrorResult {
        {
            let node: &JSRef<Node> = NodeCast::from_ref(self);
            node.wait_until_safe_to_modify_dom();
        }

        // Step 1.
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace_url));

        let name_type = xml_name_type(name.as_slice());
        match name_type {
            // Step 2.
            InvalidXMLName => return Err(InvalidCharacter),
            // Step 3.
            Name => return Err(NamespaceError),
            QName => {}
        }

        // Step 4.
        let (prefix, local_name) = get_attribute_parts(name.as_slice());
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
        if "xmlns" == name.as_slice() && namespace != namespace::XMLNS {
            return Err(NamespaceError);
        }

        // Step 8.
        if namespace == namespace::XMLNS && "xmlns" != name.as_slice() && Some("xmlns") != prefix {
            return Err(NamespaceError);
        }

        // Step 9.
        let value = self.parse_attribute(&namespace, local_name.as_slice(), value);
        self.do_set_attribute(local_name.to_string(), value, name.to_string(),
                              namespace.clone(), prefix.map(|s| s.to_string()),
                              |attr| {
            attr.deref().local_name.as_slice() == local_name &&
            attr.deref().namespace == namespace
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(&self,
                       name: DOMString) -> ErrorResult {
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };
        self.remove_attribute(namespace::Null, name.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(&self,
                         namespace: Option<DOMString>,
                         localname: DOMString) -> ErrorResult {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.remove_attribute(namespace, localname.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(&self,
                    name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(&self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    fn GetElementsByTagName(&self, localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name(&*window, NodeCast::from_ref(self), localname)
    }

    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>,
                              localname: DOMString) -> Temporary<HTMLCollection> {
        let namespace = match maybe_ns {
            Some(namespace) => Namespace::from_str(namespace.as_slice()),
            None => Null
        };
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name_ns(&*window, NodeCast::from_ref(self), localname, namespace)
    }

    fn GetElementsByClassName(&self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_class_name(&*window, NodeCast::from_ref(self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self) -> Temporary<ClientRectList> {
        let win = window_from_node(self).root();
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rects = node.get_content_boxes();
        let rects: Vec<Root<ClientRect>> = rects.iter().map(|r| {
            ClientRect::new(
                &*win,
                r.origin.y,
                r.origin.y + r.size.height,
                r.origin.x,
                r.origin.x + r.size.width).root()
        }).collect();

        ClientRectList::new(&*win, rects.iter().map(|rect| rect.deref().clone()).collect())
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self) -> Temporary<ClientRect> {
        let win = window_from_node(self).root();
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        ClientRect::new(
            &*win,
            rect.origin.y,
            rect.origin.y + rect.size.height,
            rect.origin.x,
            rect.origin.x + rect.size.width)
    }

    fn GetInnerHTML(&self) -> Fallible<DOMString> {
        //XXX TODO: XML case
        Ok(serialize(&mut NodeIterator::new(NodeCast::from_ref(self), false, false)))
    }

    fn GetOuterHTML(&self) -> Fallible<DOMString> {
        Ok(serialize(&mut NodeIterator::new(NodeCast::from_ref(self), true, false)))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(&*window, NodeCast::from_ref(self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: &JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: &JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }
}

pub fn get_attribute_parts<'a>(name: &'a str) -> (Option<&'a str>, &'a str) {
    //FIXME: Throw for XML-invalid names
    //FIXME: Throw for XMLNS-invalid names
    let (prefix, local_name) = if name.contains(":")  {
        let mut parts = name.splitn(':', 1);
        (Some(parts.next().unwrap()), parts.next().unwrap())
    } else {
        (None, name)
    };

    (prefix, local_name)
}

impl<'a> VirtualMethods for JSRef<'a, Element> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods+> {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        Some(node as &VirtualMethods+)
    }

    fn after_set_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                let doc = document_from_node(self).root();
                let base_url = doc.deref().url().clone();
                let style = Some(style::parse_style_attribute(value.as_slice(), &base_url));
                *self.deref().style_attribute.deref().borrow_mut() = style;
            }
            "id" => {
                let node: &JSRef<Node> = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    let doc = document_from_node(self).root();
                    doc.register_named_element(self, value.clone());
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn before_remove_attr(&self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name.clone(), value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                *self.deref().style_attribute.deref().borrow_mut() = None;
            }
            "id" => {
                let node: &JSRef<Node> = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    let doc = document_from_node(self).root();
                    doc.unregister_named_element(self, value);
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn parse_plain_attribute(&self, name: &str, value: DOMString) -> AttrValue {
        match name {
            "class" => AttrValue::from_tokenlist(value),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        if !tree_in_doc { return; }

        match self.get_attribute(Null, "id").root() {
            Some(attr) => {
                let doc = document_from_node(self).root();
                doc.deref().register_named_element(self, attr.deref().Value());
            }
            _ => ()
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.unbind_from_tree(tree_in_doc),
            _ => (),
        }

        if !tree_in_doc { return; }

        match self.get_attribute(Null, "id").root() {
            Some(attr) => {
                let doc = document_from_node(self).root();
                doc.deref().unregister_named_element(self, attr.deref().Value());
            }
            _ => ()
        }
    }
}

impl<'a> style::TElement for JSRef<'a, Element> {
    fn get_attr(&self, namespace: &Namespace, attr: &str) -> Option<&'static str> {
        self.get_attribute(namespace.clone(), attr).root().map(|attr| {
            unsafe { mem::transmute(attr.deref().value().as_slice()) }
        })
    }
    fn get_link(&self) -> Option<&'static str> {
        // FIXME: This is HTML only.
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId) => self.get_attr(&namespace::Null, "href"),
            _ => None,
         }
    }
    fn get_local_name<'a>(&'a self) -> &'a str {
        (self as &ElementHelpers).get_local_name()
    }
    fn get_namespace<'a>(&'a self) -> &'a Namespace {
        (self as &ElementHelpers).get_namespace()
    }
    fn get_hover_state(&self) -> bool {
        let node: &JSRef<Node> = NodeCast::from_ref(self);
        node.get_hover_state()
    }
}
