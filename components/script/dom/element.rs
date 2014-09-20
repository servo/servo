/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::attr::{Attr, ReplacedAttr, FirstSetAttr, AttrHelpers, AttrHelpersForLayout};
use dom::attr::{AttrValue, StringAttrValue, UIntAttrValue, AtomAttrValue};
use dom::namednodemap::NamedNodeMap;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::InheritTypes::{ElementDerived, NodeCast};
use dom::bindings::js::{JS, JSRef, Temporary, TemporaryPushable};
use dom::bindings::js::{OptionalSettable, OptionalRootable, Root};
use dom::bindings::trace::Traceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::error::{ErrorResult, Fallible, NamespaceError, InvalidCharacter, Syntax};
use dom::bindings::utils::{QName, Name, InvalidXMLName, xml_name_type};
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
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
use devtools_traits::AttrInfo;
use style::{matches, parse_selector_list_from_str};
use style;
use servo_util::atom::Atom;
use servo_util::namespace;
use servo_util::namespace::{Namespace, Null};
use servo_util::str::{DOMString, null_str_as_empty_ref};

use std::ascii::StrAsciiExt;
use std::cell::{Cell, RefCell};
use std::mem;

#[deriving(Encodable)]
#[must_root]
pub struct Element {
    pub node: Node,
    pub local_name: Atom,
    pub namespace: Namespace,
    pub prefix: Option<DOMString>,
    pub attrs: RefCell<Vec<JS<Attr>>>,
    pub style_attribute: Traceable<RefCell<Option<style::PropertyDeclarationBlock>>>,
    pub attr_list: Cell<Option<JS<NamedNodeMap>>>,
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
    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: JSRef<Document>) -> Element {
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

    pub fn new(local_name: DOMString, namespace: Namespace, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<Element> {
        Node::reflect_node(box Element::new_inherited(ElementTypeId, local_name, namespace, prefix, document),
                           document, ElementBinding::Wrap)
    }
}

pub trait RawLayoutElementHelpers {
    unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str) -> Option<&'static str>;
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &str) -> Option<Atom>;
    unsafe fn has_class_for_layout(&self, name: &str) -> bool;
}

impl RawLayoutElementHelpers for Element {
    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_attr_val_for_layout(&self, namespace: &Namespace, name: &str)
                                      -> Option<&'static str> {
        // cast to point to T in RefCell<T> directly
        let attrs: *const Vec<JS<Attr>> = mem::transmute(&self.attrs);
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            name == (*attr).local_name_atom_forever().as_slice() &&
            (*attr).namespace == *namespace
        }).map(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_ref_forever()
        })
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &str)
                                      -> Option<Atom> {
        // cast to point to T in RefCell<T> directly
        let attrs: *const Vec<JS<Attr>> = mem::transmute(&self.attrs);
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            name == (*attr).local_name_atom_forever().as_slice() &&
            (*attr).namespace == *namespace
        }).and_then(|attr| {
            let attr = attr.unsafe_get();
            (*attr).value_atom_forever()
        })
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn has_class_for_layout(&self, name: &str) -> bool {
        let attrs: *const Vec<JS<Attr>> = mem::transmute(&self.attrs);
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.unsafe_get();
            (*attr).local_name_atom_forever().as_slice() == "class"
        }).map_or(false, |attr| {
            let attr = attr.unsafe_get();
            (*attr).value_tokens_forever().map(|mut tokens| { tokens.any(|atom| atom.as_slice() == name) })
        }.take().unwrap())
    }
}

pub trait LayoutElementHelpers {
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
}

impl LayoutElementHelpers for JS<Element> {
    #[allow(unrooted_must_root)]
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
    fn get_local_name<'a>(&'a self) -> &'a Atom;
    fn get_namespace<'a>(&'a self) -> &'a Namespace;
    fn summarize(self) -> Vec<AttrInfo>;
    fn is_void(self) -> bool;
}

impl<'a> ElementHelpers for JSRef<'a, Element> {
    fn html_element_in_html_document(&self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        self.namespace == namespace::HTML && node.is_in_html_doc()
    }

    fn get_local_name<'a>(&'a self) -> &'a Atom {
        &self.deref().local_name
    }

    fn get_namespace<'a>(&'a self) -> &'a Namespace {
        &self.deref().namespace
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

pub trait AttributeHandlers {
    /// Returns the attribute with given namespace and case-sensitive local
    /// name, if any.
    fn get_attribute(self, namespace: Namespace, local_name: &str)
                     -> Option<Temporary<Attr>>;
    fn set_attribute_from_parser(self, local_name: Atom,
                                 value: DOMString, namespace: Namespace,
                                 prefix: Option<DOMString>);
    fn set_attribute(self, name: &str, value: AttrValue);
    fn do_set_attribute(self, local_name: Atom, value: AttrValue,
                        name: Atom, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |JSRef<Attr>| -> bool);
    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue;

    fn remove_attribute(self, namespace: Namespace, name: &str);
    fn notify_attribute_changed(self, local_name: &Atom);
    fn has_class(&self, name: &str) -> bool;

    fn set_atomic_attribute(self, name: &str, value: DOMString);

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn has_attribute(self, name: &str) -> bool;
    fn set_bool_attribute(self, name: &str, value: bool);
    fn get_url_attribute(self, name: &str) -> DOMString;
    fn set_url_attribute(self, name: &str, value: DOMString);
    fn get_string_attribute(self, name: &str) -> DOMString;
    fn set_string_attribute(self, name: &str, value: DOMString);
    fn set_tokenlist_attribute(self, name: &str, value: DOMString);
    fn get_uint_attribute(self, name: &str) -> u32;
    fn set_uint_attribute(self, name: &str, value: u32);
}

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(self, namespace: Namespace, local_name: &str) -> Option<Temporary<Attr>> {
        let local_name = Atom::from_slice(local_name);
        self.attrs.borrow().iter().map(|attr| attr.root()).find(|attr| {
            *attr.local_name() == local_name && attr.namespace == namespace
        }).map(|x| Temporary::from_rooted(*x))
    }

    fn set_attribute_from_parser(self, local_name: Atom,
                                 value: DOMString, namespace: Namespace,
                                 prefix: Option<DOMString>) {
        let name = match prefix {
            None => local_name.clone(),
            Some(ref prefix) => {
                let name = format!("{:s}:{:s}", *prefix, local_name.as_slice());
                Atom::from_slice(name.as_slice())
            },
        };
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.do_set_attribute(local_name, value, name, namespace, prefix, |_| false)
    }

    fn set_attribute(self, name: &str, value: AttrValue) {
        assert!(name == name.to_ascii_lower().as_slice());
        assert!(!name.contains(":"));

        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.wait_until_safe_to_modify_dom();

        let name = Atom::from_slice(name);
        self.do_set_attribute(name.clone(), value, name.clone(),
            namespace::Null, None, |attr| *attr.local_name() == name);
    }

    fn do_set_attribute(self, local_name: Atom, value: AttrValue,
                        name: Atom, namespace: Namespace,
                        prefix: Option<DOMString>, cb: |JSRef<Attr>| -> bool) {
        let idx = self.deref().attrs.borrow().iter()
                                    .map(|attr| attr.root())
                                    .position(|attr| cb(*attr));
        let (idx, set_type) = match idx {
            Some(idx) => (idx, ReplacedAttr),
            None => {
                let window = window_from_node(self).root();
                let attr = Attr::new(*window, local_name, value.clone(),
                                     name, namespace.clone(), prefix, self);
                self.deref().attrs.borrow_mut().push_unrooted(&attr);
                (self.deref().attrs.borrow().len() - 1, FirstSetAttr)
            }
        };

        (*self.deref().attrs.borrow())[idx].root().set_value(set_type, value);
    }

    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue {
        if *namespace == namespace::Null {
            vtable_for(&NodeCast::from_ref(self))
                .parse_plain_attribute(local_name.as_slice(), value)
        } else {
            StringAttrValue(value)
        }
    }

    fn remove_attribute(self, namespace: Namespace, name: &str) {
        let (_, local_name) = get_attribute_parts(name);
        let local_name = Atom::from_slice(local_name);

        let idx = self.deref().attrs.borrow().iter().map(|attr| attr.root()).position(|attr| {
            *attr.local_name() == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                {
                    let node: JSRef<Node> = NodeCast::from_ref(self);
                    node.wait_until_safe_to_modify_dom();
                }

                if namespace == namespace::Null {
                    let removed_raw_value = (*self.deref().attrs.borrow())[idx].root().Value();
                    vtable_for(&NodeCast::from_ref(self))
                        .before_remove_attr(&local_name,
                                            removed_raw_value);
                }

                self.deref().attrs.borrow_mut().remove(idx);
            }
        };
    }

    fn notify_attribute_changed(self, local_name: &Atom) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
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
        self.get_attribute(Null, "class").root().map(|attr| {
            attr.deref().value().tokens().map(|mut tokens| {
                tokens.any(|atom| atom.as_slice() == name)
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn set_atomic_attribute(self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower().as_slice());
        let value = AttrValue::from_atomic(value);
        self.set_attribute(name, value);
    }

    fn has_attribute(self, name: &str) -> bool {
        let name = match self.html_element_in_html_document() {
            true => Atom::from_slice(name.to_ascii_lower().as_slice()),
            false => Atom::from_slice(name)
        };
        self.deref().attrs.borrow().iter().map(|attr| attr.root()).any(|attr| {
            *attr.local_name() == name && attr.namespace == Null
        })
    }

    fn set_bool_attribute(self, name: &str, value: bool) {
        if self.has_attribute(name) == value { return; }
        if value {
            self.set_string_attribute(name, String::new());
        } else {
            self.remove_attribute(Null, name);
        }
    }

    fn get_url_attribute(self, name: &str) -> DOMString {
        assert!(name == name.to_ascii_lower().as_slice());
        // XXX Resolve URL.
        self.get_string_attribute(name)
    }
    fn set_url_attribute(self, name: &str, value: DOMString) {
        self.set_string_attribute(name, value);
    }

    fn get_string_attribute(self, name: &str) -> DOMString {
        assert!(name == name.to_ascii_lower().as_slice());
        match self.get_attribute(Null, name) {
            Some(x) => {
                let x = x.root();
                x.deref().Value()
            }
            None => "".to_string()
        }
    }
    fn set_string_attribute(self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, StringAttrValue(value));
    }

    fn set_tokenlist_attribute(self, name: &str, value: DOMString) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, AttrValue::from_tokenlist(value));
    }

    fn get_uint_attribute(self, name: &str) -> u32 {
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
    fn set_uint_attribute(self, name: &str, value: u32) {
        assert!(name == name.to_ascii_lower().as_slice());
        self.set_attribute(name, UIntAttrValue(value.to_string(), value));
    }
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(self) -> Option<DOMString> {
        match self.namespace {
            Null => None,
            ref ns => Some(ns.to_str().to_string())
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
        self.get_string_attribute("id")
    }

    // http://dom.spec.whatwg.org/#dom-element-id
    fn SetId(self, id: DOMString) {
        self.set_atomic_attribute("id", id);
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(self) -> DOMString {
        self.get_string_attribute("class")
    }

    // http://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(self, class: DOMString) {
        self.set_tokenlist_attribute("class", class);
    }

    // http://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(self) -> Temporary<DOMTokenList> {
        match self.class_list.get() {
            Some(class_list) => Temporary::new(class_list),
            None => {
                let class_list = DOMTokenList::new(self, "class").root();
                self.class_list.assign(Some(class_list.deref().clone()));
                Temporary::from_rooted(*class_list)
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(self) -> Temporary<NamedNodeMap> {
        match self.attr_list.get() {
            None => (),
            Some(ref list) => return Temporary::new(list.clone()),
        }

        let doc = {
            let node: JSRef<Node> = NodeCast::from_ref(self);
            node.owner_doc().root()
        };
        let window = doc.deref().window.root();
        let list = NamedNodeMap::new(*window, self);
        self.attr_list.assign(Some(list));
        Temporary::new(self.attr_list.get().get_ref().clone())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(self, name: DOMString) -> Option<DOMString> {
        let name = if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lower()
        } else {
            name
        };
        self.get_attribute(Null, name.as_slice()).root()
                     .map(|s| s.deref().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.get_attribute(namespace, local_name.as_slice()).root()
                     .map(|attr| attr.deref().Value())
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
        let value = self.parse_attribute(&namespace::Null, &name, value);
        self.do_set_attribute(name.clone(), value, name.clone(), namespace::Null, None, |attr| {
            attr.deref().name.as_slice() == name.as_slice()
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

        let name = Atom::from_slice(name.as_slice());
        let local_name = Atom::from_slice(local_name);
        let xmlns = Atom::from_slice("xmlns");      // TODO: Make this a static atom type

        // Step 7a.
        if xmlns == name && namespace != namespace::XMLNS {
            return Err(NamespaceError);
        }

        // Step 8.
        if namespace == namespace::XMLNS && xmlns != name && Some("xmlns") != prefix {
            return Err(NamespaceError);
        }

        // Step 9.
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.do_set_attribute(local_name.clone(), value, name,
                              namespace.clone(), prefix.map(|s| s.to_string()),
                              |attr| {
            *attr.local_name() == local_name &&
            attr.namespace == namespace
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
        self.remove_attribute(namespace::Null, name.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(self,
                         namespace: Option<DOMString>,
                         localname: DOMString) {
        let namespace = Namespace::from_str(null_str_as_empty_ref(&namespace));
        self.remove_attribute(namespace, localname.as_slice())
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(self,
                    name: DOMString) -> bool {
        self.has_attribute(name.as_slice())
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
        let mut parts = name.splitn(':', 1);
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

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                let doc = document_from_node(*self).root();
                let base_url = doc.deref().url().clone();
                let style = Some(style::parse_style_attribute(value.as_slice(), &base_url));
                *self.deref().style_attribute.deref().borrow_mut() = style;
            }
            "id" => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() && !value.is_empty() {
                    let doc = document_from_node(*self).root();
                    doc.register_named_element(*self, value.clone());
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value.clone()),
            _ => (),
        }

        match name.as_slice() {
            "style" => {
                *self.deref().style_attribute.deref().borrow_mut() = None;
            }
            "id" => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() && !value.is_empty() {
                    let doc = document_from_node(*self).root();
                    doc.unregister_named_element(*self, value);
                }
            }
            _ => ()
        }

        self.notify_attribute_changed(name);
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

        match self.get_attribute(Null, "id").root() {
            Some(attr) => {
                let doc = document_from_node(*self).root();
                let value = attr.deref().Value();
                if !value.is_empty() {
                    doc.deref().register_named_element(*self, value);
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

        match self.get_attribute(Null, "id").root() {
            Some(attr) => {
                let doc = document_from_node(*self).root();
                let value = attr.deref().Value();
                if !value.is_empty() {
                    doc.deref().unregister_named_element(*self, value);
                }
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
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match node.type_id() {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId) => self.get_attr(&namespace::Null, "href"),
            _ => None,
         }
    }
    fn get_local_name<'a>(&'a self) -> &'a Atom {
        (self as &ElementHelpers).get_local_name()
    }
    fn get_namespace<'a>(&'a self) -> &'a Namespace {
        (self as &ElementHelpers).get_namespace()
    }
    fn get_hover_state(&self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.get_hover_state()
    }
    fn get_id<'a>(&self) -> Option<Atom> {
        self.get_attribute(namespace::Null, "id").map(|attr| {
            let attr = attr.root();
            match *attr.value() {
                AtomAttrValue(ref val) => val.clone(),
                _ => fail!("`id` attribute should be AtomAttrValue"),
            }
        })
    }
    fn get_disabled_state(&self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.get_disabled_state()
    }
    fn get_enabled_state(&self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.get_enabled_state()
    }
    fn has_class(&self, name: &str) -> bool {
        (self as &AttributeHandlers).has_class(name)
    }
}
