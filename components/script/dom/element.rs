/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::{Attr, ReplacedAttr, FirstSetAttr, AttrHelpers, AttrHelpersForLayout};
use dom::attr::{AttrValue, StringAttrValue, UIntAttrValue, AtomAttrValue};
use dom::namednodemap::NamedNodeMap;
use dom::bindings::cell::{DOMRefCell, Ref, RefMut};
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::InheritTypes::{ElementDerived, HTMLInputElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableCellElementDerived, NodeCast};
use dom::bindings::js::{MutNullableJS, JS, JSRef, Temporary, TemporaryPushable};
use dom::bindings::js::{OptionalSettable, OptionalRootable, Root};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{ErrorResult, Fallible, NamespaceError, InvalidCharacter, Syntax};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::document::{Document, DocumentHelpers, LayoutDocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use dom::htmlserializer::serialize;
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementHelpers};
use dom::node::{ElementNodeTypeId, Node, NodeHelpers, NodeIterator, document_from_node};
use dom::node::{window_from_node, LayoutNodeHelpers};
use dom::nodelist::NodeList;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use devtools_traits::AttrInfo;
use style::{IntegerAttribute, LengthAttribute, SizeIntegerAttribute, WidthLengthAttribute};
use style::{matches, parse_selector_list_from_str};
use style;
use servo_util::namespace;
use servo_util::str::{DOMString, LengthOrPercentageOrAuto};

use std::ascii::StrAsciiExt;
use std::default::Default;
use std::mem;
use string_cache::{Atom, Namespace, QualName};
use url::UrlParser;

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: Atom,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    style_attribute: DOMRefCell<Option<style::PropertyDeclarationBlock>>,
    attr_list: MutNullableJS<NamedNodeMap>,
    class_list: MutNullableJS<DOMTokenList>,
}

impl ElementDerived for EventTarget {
    #[inline]
    fn is_element(&self) -> bool {
        match *self.type_id() {
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

#[deriving(PartialEq, Show)]
#[jstraceable]
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

    ElementTypeId_,
}

//
// Element methods
//

impl Element {
    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: JSRef<Document>) -> Element {
        Element {
            node: Node::new_inherited(ElementNodeTypeId(type_id), document),
            local_name: Atom::from_slice(local_name.as_slice()),
            namespace: namespace,
            prefix: prefix,
            attrs: DOMRefCell::new(vec!()),
            attr_list: Default::default(),
            class_list: Default::default(),
            style_attribute: DOMRefCell::new(None),
        }
    }

    pub fn new(local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<Element> {
        Node::reflect_node(box Element::new_inherited(ElementTypeId_, local_name, namespace, prefix, document),
                           document, ElementBinding::Wrap)
    }

    #[inline]
    pub fn node<'a>(&'a self) -> &'a Node {
        &self.node
    }

    #[inline]
    pub fn local_name<'a>(&'a self) -> &'a Atom {
        &self.local_name
    }

    #[inline]
    pub fn namespace<'a>(&'a self) -> &'a Namespace {
        &self.namespace
    }

    #[inline]
    pub fn prefix<'a>(&'a self) -> &'a Option<DOMString> {
        &self.prefix
    }

    #[inline]
    pub fn attrs(&self) -> Ref<Vec<JS<Attr>>> {
        self.attrs.borrow()
    }

    #[inline]
    pub fn attrs_mut(&self) -> RefMut<Vec<JS<Attr>>> {
        self.attrs.borrow_mut()
    }

    #[inline]
    pub fn style_attribute<'a>(&'a self) -> &'a DOMRefCell<Option<style::PropertyDeclarationBlock>> {
        &self.style_attribute
    }
}

pub trait RawLayoutElementHelpers {
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                      -> Option<&'a str>;
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str>;
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom) -> Option<Atom>;
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool;
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]>;
    unsafe fn get_length_attribute_for_layout(&self, length_attribute: LengthAttribute)
                                              -> LengthOrPercentageOrAuto;
    unsafe fn get_integer_attribute_for_layout(&self, integer_attribute: IntegerAttribute)
                                               -> Option<i32>;
}

impl RawLayoutElementHelpers for Element {
    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                          -> Option<&'a str> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            *name == (*attr).local_name_atom_forever() &&
            (*attr).namespace() == namespace
        }).map(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_ref_forever()
        })
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().filter_map(|attr: &JS<Attr>| {
            let attr = attr.unsafe_get();
            if *name == (*attr).local_name_atom_forever() {
              Some((*attr).value_ref_forever())
            } else {
              None
            }
        }).collect()
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom)
                                      -> Option<Atom> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            *name == (*attr).local_name_atom_forever() &&
            (*attr).namespace() == namespace
        }).and_then(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_atom_forever()
        })
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            (*attr).local_name_atom_forever() == atom!("class")
        }).map_or(false, |attr| {
            let attr = attr.unsafe_get();
            (*attr).value_tokens_forever().map(|tokens| {
                tokens.iter().any(|atom| atom == name)
            })
        }.take().unwrap())
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            (*attr).local_name_atom_forever() == atom!("class")
        }).and_then(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_tokens_forever()
        })
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_length_attribute_for_layout(&self, length_attribute: LengthAttribute)
                                              -> LengthOrPercentageOrAuto {
        match length_attribute {
            WidthLengthAttribute => {
                if !self.is_htmltablecellelement() {
                    fail!("I'm not a table cell!")
                }
                let this: &HTMLTableCellElement = mem::transmute(self);
                this.get_width()
            }
        }
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_integer_attribute_for_layout(&self, integer_attribute: IntegerAttribute)
                                               -> Option<i32> {
        match integer_attribute {
            SizeIntegerAttribute => {
                if !self.is_htmlinputelement() {
                    fail!("I'm not a form input!")
                }
                let this: &HTMLInputElement = mem::transmute(self);
                Some(this.get_size_for_layout() as i32)
            }
        }
    }
}

pub trait LayoutElementHelpers {
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
}

impl LayoutElementHelpers for JS<Element> {
    #[allow(unrooted_must_root)]
    #[inline]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if (*self.unsafe_get()).namespace != ns!(HTML) {
            return false
        }
        let node: JS<Node> = self.transmute_copy();
        node.owner_doc_for_layout().is_html_document_for_layout()
    }
}

pub trait ElementHelpers<'a> {
    fn html_element_in_html_document(self) -> bool;
    fn get_local_name(self) -> &'a Atom;
    fn get_namespace(self) -> &'a Namespace;
    fn summarize(self) -> Vec<AttrInfo>;
    fn is_void(self) -> bool;
}

impl<'a> ElementHelpers<'a> for JSRef<'a, Element> {
    fn html_element_in_html_document(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        self.namespace == ns!(HTML) && node.is_in_html_doc()
    }

    fn get_local_name(self) -> &'a Atom {
        &self.extended_deref().local_name
    }

    fn get_namespace(self) -> &'a Namespace {
        &self.extended_deref().namespace
    }

    fn summarize(self) -> Vec<AttrInfo> {
        let attrs = self.Attributes().root();
        let mut i = 0;
        let mut summarized = vec!();
        while i < attrs.Length() {
            let attr = attrs.Item(i).unwrap().root();
            summarized.push(attr.summarize());
            i += 1;
        }
        summarized
    }

   fn is_void(self) -> bool {
        if self.namespace != ns!(HTML) {
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

pub trait AttributeHandlers {
    /// Returns the attribute with given namespace and case-sensitive local
    /// name, if any.
    fn get_attribute(self, namespace: Namespace, local_name: &Atom)
                     -> Option<Temporary<Attr>>;
    fn get_attributes(self, local_name: &Atom)
                      -> Vec<Temporary<Attr>>;
    fn set_attribute_from_parser(self,
                                 name: QualName,
                                 value: DOMString,
                                 prefix: Option<DOMString>);
    fn set_attribute(self, name: &Atom, value: AttrValue);
    fn do_set_attribute(self, local_name: Atom, value: AttrValue,
                        name: Atom, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |JSRef<Attr>| -> bool);
    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue;

    fn remove_attribute(self, namespace: Namespace, name: &str);
    fn notify_attribute_changed(self, local_name: &Atom);
    fn has_class(&self, name: &Atom) -> bool;
    fn notify_attribute_removed(self);

    fn set_atomic_attribute(self, name: &Atom, value: DOMString);

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn has_attribute(self, name: &Atom) -> bool;
    fn set_bool_attribute(self, name: &Atom, value: bool);
    fn get_url_attribute(self, name: &Atom) -> DOMString;
    fn set_url_attribute(self, name: &Atom, value: DOMString);
    fn get_string_attribute(self, name: &Atom) -> DOMString;
    fn set_string_attribute(self, name: &Atom, value: DOMString);
    fn set_tokenlist_attribute(self, name: &Atom, value: DOMString);
    fn get_uint_attribute(self, name: &Atom) -> u32;
    fn set_uint_attribute(self, name: &Atom, value: u32);
}

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(self, namespace: Namespace, local_name: &Atom) -> Option<Temporary<Attr>> {
        self.get_attributes(local_name).iter().map(|attr| attr.root())
            .find(|attr| *attr.namespace() == namespace)
            .map(|x| Temporary::from_rooted(*x))
    }

    fn get_attributes(self, local_name: &Atom) -> Vec<Temporary<Attr>> {
        self.attrs.borrow().iter().map(|attr| attr.root()).filter_map(|attr| {
            if *attr.local_name() == *local_name {
                Some(Temporary::from_rooted(*attr))
            } else {
                None
            }
        }).collect()
    }

    fn set_attribute_from_parser(self,
                                 qname: QualName,
                                 value: DOMString,
                                 prefix: Option<DOMString>) {
        // Don't set if the attribute already exists, so we can handle add_attrs_if_missing
        if self.attrs.borrow().iter().map(|attr| attr.root())
                .any(|a| *a.local_name() == qname.local && *a.namespace() == qname.ns) {
            return;
        }

        let name = match prefix {
            None => qname.local.clone(),
            Some(ref prefix) => {
                let name = format!("{:s}:{:s}", *prefix, qname.local.as_slice());
                Atom::from_slice(name.as_slice())
            },
        };
        let value = self.parse_attribute(&qname.ns, &qname.local, value);
        self.do_set_attribute(qname.local, value, name, qname.ns, prefix, |_| false)
    }

    fn set_attribute(self, name: &Atom, value: AttrValue) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lower().as_slice());
        assert!(!name.as_slice().contains(":"));

        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.wait_until_safe_to_modify_dom();

        self.do_set_attribute(name.clone(), value, name.clone(),
            ns!(""), None, |attr| *attr.local_name() == *name);
    }

    fn do_set_attribute(self, local_name: Atom, value: AttrValue,
                        name: Atom, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |JSRef<Attr>| -> bool) {
        let idx = self.attrs.borrow().iter()
                                     .map(|attr| attr.root())
                                     .position(|attr| cb(*attr));
        let (idx, set_type) = match idx {
            Some(idx) => (idx, ReplacedAttr),
            None => {
                let window = window_from_node(self).root();
                let attr = Attr::new(*window, local_name, value.clone(),
                                     name, namespace.clone(), prefix, self);
                self.attrs.borrow_mut().push_unrooted(&attr);
                (self.attrs.borrow().len() - 1, FirstSetAttr)
            }
        };

        (*self.attrs.borrow())[idx].root().set_value(set_type, value);
    }

    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue {
        if *namespace == ns!("") {
            vtable_for(&NodeCast::from_ref(self))
                .parse_plain_attribute(local_name.as_slice(), value)
        } else {
            StringAttrValue(value)
        }
    }

    fn remove_attribute(self, namespace: Namespace, name: &str) {
        let (_, local_name) = get_attribute_parts(name);
        let local_name = Atom::from_slice(local_name);

        let idx = self.attrs.borrow().iter().map(|attr| attr.root()).position(|attr| {
            *attr.local_name() == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                {
                    let node: JSRef<Node> = NodeCast::from_ref(self);
                    node.wait_until_safe_to_modify_dom();
                }

                if namespace == ns!("") {
                    let attr = (*self.attrs.borrow())[idx].root();
                    vtable_for(&NodeCast::from_ref(self)).before_remove_attr(*attr);
                }

                self.attrs.borrow_mut().remove(idx);
                self.notify_attribute_removed();
            }
        };
    }

    fn notify_attribute_changed(self, _local_name: &Atom) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        if node.is_in_doc() {
            debug!("Attribute changed. Dispatching content_changed.");
            let document = node.owner_doc().root();
            document.content_changed(node);
        }
    }

    fn notify_attribute_removed(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        if node.is_in_doc() {
            let document = node.owner_doc().root();
            document.content_changed(node);
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        self.get_attribute(ns!(""), &atom!("class")).root().map(|attr| {
            attr.value().tokens().map(|tokens| {
                tokens.iter().any(|atom| atom == name)
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn set_atomic_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice().eq_ignore_ascii_case(name.as_slice()));
        let value = AttrValue::from_atomic(value);
        self.set_attribute(name, value);
    }

    fn has_attribute(self, name: &Atom) -> bool {
        assert!(name.as_slice().chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii().to_lowercase() == ch.to_ascii()
        }));
        self.attrs.borrow().iter().map(|attr| attr.root()).any(|attr| {
            *attr.local_name() == *name && *attr.namespace() == ns!("")
        })
    }

    fn set_bool_attribute(self, name: &Atom, value: bool) {
        if self.has_attribute(name) == value { return; }
        if value {
            self.set_string_attribute(name, String::new());
        } else {
            self.remove_attribute(ns!(""), name.as_slice());
        }
    }

    fn get_url_attribute(self, name: &Atom) -> DOMString {
        assert!(name.as_slice() == name.as_slice().to_ascii_lower().as_slice());
        if !self.has_attribute(name) {
            return "".to_string();
        }
        let url = self.get_string_attribute(name);
        let doc = document_from_node(self).root();
        let base = doc.url();
        // https://html.spec.whatwg.org/multipage/infrastructure.html#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match UrlParser::new().base_url(base).parse(url.as_slice()) {
            Ok(parsed) => parsed.serialize(),
            Err(_) => "".to_string()
        }
    }
    fn set_url_attribute(self, name: &Atom, value: DOMString) {
        self.set_string_attribute(name, value);
    }

    fn get_string_attribute(self, name: &Atom) -> DOMString {
        match self.get_attribute(ns!(""), name) {
            Some(x) => x.root().Value(),
            None => "".to_string()
        }
    }
    fn set_string_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lower().as_slice());
        self.set_attribute(name, StringAttrValue(value));
    }

    fn set_tokenlist_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lower().as_slice());
        self.set_attribute(name, AttrValue::from_tokenlist(value));
    }

    fn get_uint_attribute(self, name: &Atom) -> u32 {
        assert!(name.as_slice().chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii().to_lowercase() == ch.to_ascii()
        }));
        let attribute = self.get_attribute(ns!(""), name).root();
        match attribute {
            Some(attribute) => {
                match *attribute.value() {
                    UIntAttrValue(_, value) => value,
                    _ => fail!("Expected a UIntAttrValue"),
                }
            }
            None => 0,
        }
    }
    fn set_uint_attribute(self, name: &Atom, value: u32) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lower().as_slice());
        self.set_attribute(name, UIntAttrValue(value.to_string(), value));
    }
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(self) -> Option<DOMString> {
        match self.namespace {
            ns!("") => None,
            Namespace(ref ns) => Some(ns.as_slice().to_string())
        }
    }

    fn LocalName(self) -> DOMString {
        self.local_name.as_slice().to_string()
    }

    // http://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(self) -> DOMString {
        let qualified_name = match self.prefix {
            Some(ref prefix) => format!("{}:{}", prefix, self.local_name).into_maybe_owned(),
            None => self.local_name.as_slice().into_maybe_owned()
        };
        if self.html_element_in_html_document() {
            qualified_name.as_slice().to_ascii_upper()
        } else {
            qualified_name.into_string()
        }
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn Id(self) -> DOMString {
        self.get_string_attribute(&atom!("id"))
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn SetId(self, id: DOMString) {
        self.set_atomic_attribute(&atom!("id"), id);
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(self) -> DOMString {
        self.get_string_attribute(&atom!("class"))
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(self, class: DOMString) {
        self.set_tokenlist_attribute(&atom!("class"), class);
    }

    // http://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(self) -> Temporary<DOMTokenList> {
        if self.class_list.get().is_none() {
            let class_list = DOMTokenList::new(self, &atom!("class"));
            self.class_list.assign(Some(class_list));
        }
        self.class_list.get().unwrap()
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(self) -> Temporary<NamedNodeMap> {
        if self.attr_list.get().is_none() {
            let doc = {
                let node: JSRef<Node> = NodeCast::from_ref(self);
                node.owner_doc().root()
            };
            let window = doc.window().root();
            let list = NamedNodeMap::new(*window, self);
            self.attr_list.assign(Some(list));
        }
        self.attr_list.get().unwrap()
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(self, name: DOMString) -> Option<DOMString> {
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };
        self.get_attribute(ns!(""), &Atom::from_slice(name.as_slice())).root()
                     .map(|s| s.Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = namespace::from_domstring(namespace);
        self.get_attribute(namespace, &Atom::from_slice(local_name.as_slice())).root()
                     .map(|attr| attr.Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(self,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        {
            let node: JSRef<Node> = NodeCast::from_ref(self);
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
        let name = Atom::from_slice(name.as_slice());
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.do_set_attribute(name.clone(), value, name.clone(), ns!(""), None, |attr| {
            attr.name().as_slice() == name.as_slice()
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(self,
                      namespace_url: Option<DOMString>,
                      name: DOMString,
                      value: DOMString) -> ErrorResult {
        {
            let node: JSRef<Node> = NodeCast::from_ref(self);
            node.wait_until_safe_to_modify_dom();
        }

        // Step 1.
        let namespace = namespace::from_domstring(namespace_url);

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
                if namespace == ns!("") {
                    return Err(NamespaceError);
                }

                // Step 6.
                if "xml" == prefix_str.as_slice() && namespace != ns!(XML) {
                    return Err(NamespaceError);
                }

                // Step 7b.
                if "xmlns" == prefix_str.as_slice() && namespace != ns!(XMLNS) {
                    return Err(NamespaceError);
                }
            },
            None => {}
        }

        let name = Atom::from_slice(name.as_slice());
        let local_name = Atom::from_slice(local_name);
        let xmlns = Atom::from_slice("xmlns");      // TODO: Make this a static atom type

        // Step 7a.
        if xmlns == name && namespace != ns!(XMLNS) {
            return Err(NamespaceError);
        }

        // Step 8.
        if namespace == ns!(XMLNS) && xmlns != name && Some("xmlns") != prefix {
            return Err(NamespaceError);
        }

        // Step 9.
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.do_set_attribute(local_name.clone(), value, name,
                              namespace.clone(), prefix.map(|s| s.to_string()),
                              |attr| {
            *attr.local_name() == local_name &&
            *attr.namespace() == namespace
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(self, name: DOMString) {
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };
        self.remove_attribute(ns!(""), name.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(self,
                         namespace: Option<DOMString>,
                         localname: DOMString) {
        let namespace = namespace::from_domstring(namespace);
        self.remove_attribute(namespace, localname.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(self, name: DOMString) -> bool {
        // Step 1.
        if self.html_element_in_html_document() {
            // TODO(pcwalton): Small string optimization here.
            return self.has_attribute(&Atom::from_slice(name.as_slice()
                                                            .to_ascii_lower()
                                                            .as_slice()))
        }

        // Step 2.
        self.has_attribute(&Atom::from_slice(name.as_slice()))
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    fn GetElementsByTagName(self, localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name(*window, NodeCast::from_ref(self), localname)
    }

    fn GetElementsByTagNameNS(self, maybe_ns: Option<DOMString>,
                              localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name_ns(*window, NodeCast::from_ref(self), localname, maybe_ns)
    }

    fn GetElementsByClassName(self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_class_name(*window, NodeCast::from_ref(self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    fn GetClientRects(self) -> Temporary<DOMRectList> {
        let win = window_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rects = node.get_content_boxes();
        let rects: Vec<Root<DOMRect>> = rects.iter().map(|r| {
            DOMRect::new(
                *win,
                r.origin.y,
                r.origin.y + r.size.height,
                r.origin.x,
                r.origin.x + r.size.width).root()
        }).collect();

        DOMRectList::new(*win, rects.iter().map(|rect| rect.deref().clone()).collect())
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(self) -> Temporary<DOMRect> {
        let win = window_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        DOMRect::new(
            *win,
            rect.origin.y,
            rect.origin.y + rect.size.height,
            rect.origin.x,
            rect.origin.x + rect.size.width)
    }

    fn GetInnerHTML(self) -> Fallible<DOMString> {
        //XXX TODO: XML case
        Ok(serialize(&mut NodeIterator::new(NodeCast::from_ref(self), false, false)))
    }

    fn GetOuterHTML(self) -> Fallible<DOMString> {
        Ok(serialize(&mut NodeIterator::new(NodeCast::from_ref(self), true, false)))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(*window, NodeCast::from_ref(self))
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // http://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }

    // http://dom.spec.whatwg.org/#dom-element-matches
    fn Matches(self, selectors: DOMString) -> Fallible<bool> {
        match parse_selector_list_from_str(selectors.as_slice()) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root: JSRef<Node> = NodeCast::from_ref(self);
                Ok(matches(selectors, &root, &mut None))
            }
        }
    }
}

pub fn get_attribute_parts<'a>(name: &'a str) -> (Option<&'a str>, &'a str) {
    //FIXME: Throw for XML-invalid names
    //FIXME: Throw for XMLNS-invalid names
    let (prefix, local_name) = if name.contains(":")  {
        let mut parts = name.splitn(1, ':');
        (Some(parts.next().unwrap()), parts.next().unwrap())
    } else {
        (None, name)
    };

    (prefix, local_name)
}

impl<'a> VirtualMethods for JSRef<'a, Element> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let node: &JSRef<Node> = NodeCast::from_borrowed_ref(self);
        Some(node as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("style") => {
                let doc = document_from_node(*self).root();
                let base_url = doc.url().clone();
                let value = attr.value();
                let style = Some(style::parse_style_attribute(value.as_slice(), &base_url));
                *self.style_attribute.borrow_mut() = style;
            }
            &atom!("id") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                let value = attr.value();
                if node.is_in_doc() && !value.as_slice().is_empty() {
                    let doc = document_from_node(*self).root();
                    let value = Atom::from_slice(value.as_slice());
                    doc.register_named_element(*self, value);
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(attr.local_name());
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("style") => {
                *self.style_attribute.borrow_mut() = None;
            }
            &atom!("id") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                let value = attr.value();
                if node.is_in_doc() && !value.as_slice().is_empty() {
                    let doc = document_from_node(*self).root();
                    let value = Atom::from_slice(value.as_slice());
                    doc.unregister_named_element(*self, value);
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(attr.local_name());
    }

    fn parse_plain_attribute(&self, name: &str, value: DOMString) -> AttrValue {
        match name {
            "id" => AttrValue::from_atomic(value),
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

        match self.get_attribute(ns!(""), &atom!("id")).root() {
            Some(attr) => {
                let doc = document_from_node(*self).root();
                let value = attr.Value();
                if !value.is_empty() {
                    let value = Atom::from_slice(value.as_slice());
                    doc.register_named_element(*self, value);
                }
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

        match self.get_attribute(ns!(""), &atom!("id")).root() {
            Some(attr) => {
                let doc = document_from_node(*self).root();
                let value = attr.Value();
                if !value.is_empty() {
                    let value = Atom::from_slice(value.as_slice());
                    doc.unregister_named_element(*self, value);
                }
            }
            _ => ()
        }
    }
}

impl<'a> style::TElement<'a> for JSRef<'a, Element> {
    fn get_attr(self, namespace: &Namespace, attr: &Atom) -> Option<&'a str> {
        self.get_attribute(namespace.clone(), attr).root().map(|attr| {
            // This transmute is used to cheat the lifetime restriction.
            unsafe { mem::transmute(attr.value().as_slice()) }
        })
    }
    fn get_attrs(self, attr: &Atom) -> Vec<&'a str> {
        self.get_attributes(attr).iter().map(|attr| attr.root()).map(|attr| {
            // This transmute is used to cheat the lifetime restriction.
            unsafe { mem::transmute(attr.value().as_slice()) }
        }).collect()
    }
    fn get_link(self) -> Option<&'a str> {
        // FIXME: This is HTML only.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId) => self.get_attr(&ns!(""), &atom!("href")),
            _ => None,
         }
    }
    fn get_local_name(self) -> &'a Atom {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn get_local_name<'a, T: ElementHelpers<'a>>(this: T) -> &'a Atom {
            this.get_local_name()
        }

        get_local_name(self)
    }
    fn get_namespace(self) -> &'a Namespace {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn get_namespace<'a, T: ElementHelpers<'a>>(this: T) -> &'a Namespace {
            this.get_namespace()
        }

        get_namespace(self)
    }
    fn get_hover_state(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_hover_state()
    }
    fn get_id(self) -> Option<Atom> {
        self.get_attribute(ns!(""), &atom!("id")).map(|attr| {
            let attr = attr.root();
            match *attr.value() {
                AtomAttrValue(ref val) => val.clone(),
                _ => fail!("`id` attribute should be AtomAttrValue"),
            }
        })
    }
    fn get_disabled_state(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_disabled_state()
    }
    fn get_enabled_state(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_enabled_state()
    }
    fn has_class(self, name: &Atom) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn has_class<T: AttributeHandlers>(this: T, name: &Atom) -> bool {
            this.has_class(name)
        }

        has_class(self, name)
    }
    fn each_class(self, callback: |&Atom|) {
        match self.get_attribute(ns!(""), &atom!("class")).root() {
            None => {}
            Some(ref attr) => {
                match attr.value().tokens() {
                    None => {}
                    Some(tokens) => {
                        for token in tokens.iter() {
                            callback(token)
                        }
                    }
                }
            }
        }
    }
}
