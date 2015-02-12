/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::activation::Activatable;
use dom::attr::{Attr, AttrSettingType, AttrHelpers, AttrHelpersForLayout};
use dom::attr::AttrValue;
use dom::namednodemap::NamedNodeMap;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, ElementDerived, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLInputElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLTableElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, HTMLTableCellElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableRowElementDerived, HTMLTextAreaElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableSectionElementDerived, NodeCast};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{NamespaceError, InvalidCharacter, Syntax};
use dom::bindings::js::{MutNullableJS, JS, JSRef, LayoutJS, Temporary, TemporaryPushable};
use dom::bindings::js::{OptionalRootable, Root};
use dom::bindings::utils::xml_name_type;
use dom::bindings::utils::XMLName::{QName, Name, InvalidXMLName};
use dom::create::create_element;
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::document::{Document, DocumentHelpers, LayoutDocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::event::{Event, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementHelpers};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElementTypeId;
use dom::htmlinputelement::{HTMLInputElement, RawLayoutHTMLInputElementHelpers, HTMLInputElementHelpers};
use dom::htmlserializer::serialize;
use dom::htmltableelement::{HTMLTableElement, HTMLTableElementHelpers};
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementHelpers};
use dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementHelpers};
use dom::htmltablesectionelement::{HTMLTableSectionElement, HTMLTableSectionElementHelpers};
use dom::htmltextareaelement::{HTMLTextAreaElement, RawLayoutHTMLTextAreaElementHelpers};
use dom::node::{CLICK_IN_PROGRESS, LayoutNodeHelpers, Node, NodeHelpers, NodeTypeId};
use dom::node::{NodeIterator, document_from_node, NodeDamage};
use dom::node::{window_from_node};
use dom::nodelist::NodeList;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use devtools_traits::AttrInfo;
use style::legacy::{SimpleColorAttribute, UnsignedIntegerAttribute, IntegerAttribute, LengthAttribute};
use style::selector_matching::matches;
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration, parse_style_attribute};
use style::selectors::parse_author_origin_selector_list_from_str;
use style;
use util::namespace;
use util::str::{DOMString, LengthOrPercentageOrAuto};

use html5ever::tree_builder::{NoQuirks, LimitedQuirks, Quirks};

use cssparser::RGBA;
use std::ascii::AsciiExt;
use std::borrow::{IntoCow, ToOwned};
use std::cell::{Ref, RefMut};
use std::default::Default;
use std::mem;
use std::sync::Arc;
use string_cache::{Atom, Namespace, QualName};
use url::UrlParser;

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: Atom,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    style_attribute: DOMRefCell<Option<PropertyDeclarationBlock>>,
    attr_list: MutNullableJS<NamedNodeMap>,
    class_list: MutNullableJS<DOMTokenList>,
}

impl ElementDerived for EventTarget {
    #[inline]
    fn is_element(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(NodeTypeId::Element(_)) => true,
            _ => false
        }
    }
}

#[derive(Copy, PartialEq, Debug)]
#[jstraceable]
pub enum ElementTypeId {
    HTMLElement(HTMLElementTypeId),
    Element,
}

#[derive(PartialEq)]
pub enum ElementCreator {
    ParserCreated,
    ScriptCreated,
}

//
// Element methods
//
impl Element {
    pub fn create(name: QualName, prefix: Option<DOMString>,
                  document: JSRef<Document>, creator: ElementCreator)
                  -> Temporary<Element> {
        create_element(name, prefix, document, creator)
    }

    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: JSRef<Document>) -> Element {
        Element {
            node: Node::new_inherited(NodeTypeId::Element(type_id), document),
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
        Node::reflect_node(box Element::new_inherited(ElementTypeId::Element, local_name, namespace, prefix, document),
                           document, ElementBinding::Wrap)
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
    unsafe fn get_checked_state_for_layout(&self) -> bool;
    unsafe fn get_indeterminate_state_for_layout(&self) -> bool;
    unsafe fn get_unsigned_integer_attribute_for_layout(&self, attribute: UnsignedIntegerAttribute)
                                                        -> Option<u32>;
    unsafe fn get_simple_color_attribute_for_layout(&self, attribute: SimpleColorAttribute)
                                                    -> Option<RGBA>;
    fn local_name<'a>(&'a self) -> &'a Atom;
    fn namespace<'a>(&'a self) -> &'a Namespace;
    fn style_attribute<'a>(&'a self) -> &'a DOMRefCell<Option<PropertyDeclarationBlock>>;
}

#[inline]
unsafe fn get_attr_for_layout<'a>(elem: &'a Element, namespace: &Namespace, name: &Atom) -> Option<&'a JS<Attr>> {
    // cast to point to T in RefCell<T> directly
    let attrs = elem.attrs.borrow_for_layout();
    attrs.iter().find(|attr: & &JS<Attr>| {
        let attr = attr.to_layout().unsafe_get();
        *name == (*attr).local_name_atom_forever() &&
        (*attr).namespace() == namespace
    })
}

impl RawLayoutElementHelpers for Element {
    #[inline]
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                          -> Option<&'a str> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).value_ref_forever()
        })
    }

    #[inline]
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().filter_map(|attr: &JS<Attr>| {
            let attr = attr.to_layout().unsafe_get();
            if *name == (*attr).local_name_atom_forever() {
              Some((*attr).value_ref_forever())
            } else {
              None
            }
        }).collect()
    }

    #[inline]
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom)
                                      -> Option<Atom> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.to_layout().unsafe_get();
            *name == (*attr).local_name_atom_forever() &&
            (*attr).namespace() == namespace
        }).and_then(|attr| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).value_atom_forever()
        })
    }

    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).local_name_atom_forever() == atom!("class")
        }).map_or(false, |attr| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).value_tokens_forever().map(|tokens| {
                tokens.iter().any(|atom| atom == name)
            })
        }.take().unwrap())
    }

    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().find(|attr: & &JS<Attr>| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).local_name_atom_forever() == atom!("class")
        }).and_then(|attr| {
            let attr = attr.to_layout().unsafe_get();
            (*attr).value_tokens_forever()
        })
    }

    #[inline]
    unsafe fn get_length_attribute_for_layout(&self, length_attribute: LengthAttribute)
                                              -> LengthOrPercentageOrAuto {
        match length_attribute {
            LengthAttribute::Width => {
                if self.is_htmltableelement() {
                    let this: &HTMLTableElement = mem::transmute(self);
                    this.get_width()
                } else if self.is_htmltablecellelement() {
                    let this: &HTMLTableCellElement = mem::transmute(self);
                    this.get_width()
                } else {
                    panic!("I'm not a table or table cell!")
                }
            }
        }
    }

    #[inline]
    unsafe fn get_integer_attribute_for_layout(&self, integer_attribute: IntegerAttribute)
                                               -> Option<i32> {
        match integer_attribute {
            IntegerAttribute::Size => {
                if !self.is_htmlinputelement() {
                    panic!("I'm not a form input!")
                }
                let this: &HTMLInputElement = mem::transmute(self);
                Some(this.get_size_for_layout() as i32)
            }
            IntegerAttribute::Cols => {
                if !self.is_htmltextareaelement() {
                    panic!("I'm not a textarea element!")
                }
                let this: &HTMLTextAreaElement = mem::transmute(self);
                Some(this.get_cols_for_layout() as i32)
            }
            IntegerAttribute::Rows => {
                if !self.is_htmltextareaelement() {
                    panic!("I'm not a textarea element!")
                }
                let this: &HTMLTextAreaElement = mem::transmute(self);
                Some(this.get_rows_for_layout() as i32)
            }
        }
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_checked_state_for_layout(&self) -> bool {
        // TODO option and menuitem can also have a checked state.
        if !self.is_htmlinputelement() {
            return false
        }
        let this: &HTMLInputElement = mem::transmute(self);
        this.get_checked_state_for_layout()
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_indeterminate_state_for_layout(&self) -> bool {
        // TODO progress elements can also be matched with :indeterminate
        if !self.is_htmlinputelement() {
            return false
        }
        let this: &HTMLInputElement = mem::transmute(self);
        this.get_indeterminate_state_for_layout()
    }


    unsafe fn get_unsigned_integer_attribute_for_layout(&self,
                                                        attribute: UnsignedIntegerAttribute)
                                                        -> Option<u32> {
        match attribute {
            UnsignedIntegerAttribute::Border => {
                if self.is_htmltableelement() {
                    let this: &HTMLTableElement = mem::transmute(self);
                    this.get_border()
                } else {
                    // Don't panic since `:-servo-nonzero-border` can cause this to be called on
                    // arbitrary elements.
                    None
                }
            }
            UnsignedIntegerAttribute::ColSpan => {
                if self.is_htmltablecellelement() {
                    let this: &HTMLTableCellElement = mem::transmute(self);
                    this.get_colspan()
                } else {
                    // Don't panic since `display` can cause this to be called on arbitrary
                    // elements.
                    None
                }
            }
        }
    }

    #[inline]
    #[allow(unrooted_must_root)]
    unsafe fn get_simple_color_attribute_for_layout(&self, attribute: SimpleColorAttribute)
                                                    -> Option<RGBA> {
        match attribute {
            SimpleColorAttribute::BgColor => {
                if self.is_htmlbodyelement() {
                    let this: &HTMLBodyElement = mem::transmute(self);
                    this.get_background_color()
                } else if self.is_htmltableelement() {
                    let this: &HTMLTableElement = mem::transmute(self);
                    this.get_background_color()
                } else if self.is_htmltablecellelement() {
                    let this: &HTMLTableCellElement = mem::transmute(self);
                    this.get_background_color()
                } else if self.is_htmltablerowelement() {
                    let this: &HTMLTableRowElement = mem::transmute(self);
                    this.get_background_color()
                } else if self.is_htmltablesectionelement() {
                    let this: &HTMLTableSectionElement = mem::transmute(self);
                    this.get_background_color()
                } else {
                    None
                }
            }
        }
    }

    // Getters used in components/layout/wrapper.rs

    fn local_name<'a>(&'a self) -> &'a Atom {
        &self.local_name
    }

    fn namespace<'a>(&'a self) -> &'a Namespace {
        &self.namespace
    }

    fn style_attribute<'a>(&'a self) -> &'a DOMRefCell<Option<PropertyDeclarationBlock>> {
        &self.style_attribute
    }
}

pub trait LayoutElementHelpers {
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    unsafe fn has_attr_for_layout(&self, namespace: &Namespace, name: &Atom) -> bool;
}

impl LayoutElementHelpers for LayoutJS<Element> {
    #[inline]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if (*self.unsafe_get()).namespace != ns!(HTML) {
            return false
        }
        let node: LayoutJS<Node> = self.transmute_copy();
        node.owner_doc_for_layout().is_html_document_for_layout()
    }

    unsafe fn has_attr_for_layout(&self, namespace: &Namespace, name: &Atom) -> bool {
        get_attr_for_layout(&*self.unsafe_get(), namespace, name).is_some()
    }
}

#[derive(PartialEq)]
pub enum StylePriority {
    Important,
    Normal,
}

pub trait ElementHelpers<'a> {
    fn html_element_in_html_document(self) -> bool;
    fn local_name(self) -> &'a Atom;
    fn parsed_name(self, name: DOMString) -> DOMString;
    fn namespace(self) -> &'a Namespace;
    fn prefix(self) -> &'a Option<DOMString>;
    fn attrs(&self) -> Ref<Vec<JS<Attr>>>;
    fn attrs_mut(&self) -> RefMut<Vec<JS<Attr>>>;
    fn style_attribute(self) -> &'a DOMRefCell<Option<PropertyDeclarationBlock>>;
    fn summarize(self) -> Vec<AttrInfo>;
    fn is_void(self) -> bool;
    fn remove_inline_style_property(self, property: DOMString);
    fn update_inline_style(self, property_decl: PropertyDeclaration, style_priority: StylePriority);
    fn get_inline_style_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
    fn get_important_inline_style_declaration(self, property: &Atom) -> Option<PropertyDeclaration>;
}

impl<'a> ElementHelpers<'a> for JSRef<'a, Element> {
    fn html_element_in_html_document(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        self.namespace == ns!(HTML) && node.is_in_html_doc()
    }

    fn local_name(self) -> &'a Atom {
        &self.extended_deref().local_name
    }

    // https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name
    fn parsed_name(self, name: DOMString) -> DOMString {
        if self.html_element_in_html_document() {
            name.as_slice().to_ascii_lowercase()
        } else {
            name
        }
    }

    fn namespace(self) -> &'a Namespace {
        &self.extended_deref().namespace
    }

    fn prefix(self) -> &'a Option<DOMString> {
        &self.extended_deref().prefix
    }

    fn attrs(&self) -> Ref<Vec<JS<Attr>>> {
        self.extended_deref().attrs.borrow()
    }

    fn attrs_mut(&self) -> RefMut<Vec<JS<Attr>>> {
        self.extended_deref().attrs.borrow_mut()
    }

    fn style_attribute(self) -> &'a DOMRefCell<Option<PropertyDeclarationBlock>> {
        &self.extended_deref().style_attribute
    }

    fn summarize(self) -> Vec<AttrInfo> {
        let attrs = self.Attributes().root();
        let mut i = 0;
        let mut summarized = vec!();
        while i < attrs.r().Length() {
            let attr = attrs.r().Item(i).unwrap().root();
            summarized.push(attr.r().summarize());
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

    fn remove_inline_style_property(self, property: DOMString) {
        let mut inline_declarations = self.style_attribute.borrow_mut();
        inline_declarations.as_mut().map(|declarations| {
            let index = declarations.normal
                                    .iter()
                                    .position(|decl| decl.name() == property);
            match index {
                Some(index) => {
                    declarations.normal.make_unique().remove(index);
                    return;
                }
                None => ()
            }

            let index = declarations.important
                                    .iter()
                                    .position(|decl| decl.name() == property);
            match index {
                Some(index) => {
                    declarations.important.make_unique().remove(index);
                    return;
                }
                None => ()
            }
        });
    }

    fn update_inline_style(self, property_decl: PropertyDeclaration, style_priority: StylePriority) {
        let mut inline_declarations = self.style_attribute().borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let existing_declarations = if style_priority == StylePriority::Important {
                declarations.important.make_unique()
            } else {
                declarations.normal.make_unique()
            };

            for declaration in existing_declarations.iter_mut() {
                if declaration.name() == property_decl.name() {
                    *declaration = property_decl;
                    return;
                }
            }
            existing_declarations.push(property_decl);
            return;
        }

        let (important, normal) = if style_priority == StylePriority::Important {
            (vec!(property_decl), vec!())
        } else {
            (vec!(), vec!(property_decl))
        };

        *inline_declarations = Some(PropertyDeclarationBlock {
            important: Arc::new(important),
            normal: Arc::new(normal),
        });
    }

    fn get_inline_style_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let inline_declarations = self.style_attribute.borrow();
        inline_declarations.as_ref().and_then(|declarations| {
            declarations.normal
                        .iter()
                        .chain(declarations.important.iter())
                        .find(|decl| decl.matches(property.as_slice()))
                        .map(|decl| decl.clone())
        })
    }

    fn get_important_inline_style_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let inline_declarations = self.style_attribute.borrow();
        inline_declarations.as_ref().and_then(|declarations| {
            declarations.important
                        .iter()
                        .find(|decl| decl.matches(property.as_slice()))
                        .map(|decl| decl.clone())
        })
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
    fn set_custom_attribute(self, name: DOMString, value: DOMString) -> ErrorResult;
    fn do_set_attribute<F>(self, local_name: Atom, value: AttrValue,
                           name: Atom, namespace: Namespace,
                           prefix: Option<DOMString>, cb: F)
        where F: Fn(JSRef<Attr>) -> bool;
    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue;

    fn remove_attribute(self, namespace: Namespace, name: &str);
    fn has_class(self, name: &Atom) -> bool;

    fn set_atomic_attribute(self, name: &Atom, value: DOMString);

    // http://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn has_attribute(self, name: &Atom) -> bool;
    fn set_bool_attribute(self, name: &Atom, value: bool);
    fn get_url_attribute(self, name: &Atom) -> DOMString;
    fn set_url_attribute(self, name: &Atom, value: DOMString);
    fn get_string_attribute(self, name: &Atom) -> DOMString;
    fn set_string_attribute(self, name: &Atom, value: DOMString);
    fn get_tokenlist_attribute(self, name: &Atom) -> Vec<Atom>;
    fn set_tokenlist_attribute(self, name: &Atom, value: DOMString);
    fn set_atomic_tokenlist_attribute(self, name: &Atom, tokens: Vec<Atom>);
    fn get_uint_attribute(self, name: &Atom) -> u32;
    fn set_uint_attribute(self, name: &Atom, value: u32);
}

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(self, namespace: Namespace, local_name: &Atom) -> Option<Temporary<Attr>> {
        self.get_attributes(local_name).into_iter().map(|attr| attr.root())
            .find(|attr| *attr.r().namespace() == namespace)
            .map(|x| Temporary::from_rooted(x.r()))
    }

    fn get_attributes(self, local_name: &Atom) -> Vec<Temporary<Attr>> {
        self.attrs.borrow().iter().map(|attr| attr.root()).filter_map(|attr| {
            if *attr.r().local_name() == *local_name {
                Some(Temporary::from_rooted(attr.r()))
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
                .any(|a| *a.r().local_name() == qname.local && *a.r().namespace() == qname.ns) {
            return;
        }

        let name = match prefix {
            None => qname.local.clone(),
            Some(ref prefix) => {
                let name = format!("{}:{}", *prefix, qname.local.as_slice());
                Atom::from_slice(name.as_slice())
            },
        };
        let value = self.parse_attribute(&qname.ns, &qname.local, value);
        self.do_set_attribute(qname.local, value, name, qname.ns, prefix, |_| false)
    }

    fn set_attribute(self, name: &Atom, value: AttrValue) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        assert!(!name.as_slice().contains(":"));

        self.do_set_attribute(name.clone(), value, name.clone(),
            ns!(""), None, |attr| *attr.local_name() == *name);
    }

    // https://html.spec.whatwg.org/multipage/dom.html#attr-data-*
    fn set_custom_attribute(self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        match xml_name_type(name.as_slice()) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Steps 2-5.
        let name = Atom::from_slice(name.as_slice());
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.do_set_attribute(name.clone(), value, name.clone(), ns!(""), None, |attr| {
            *attr.name() == name && *attr.namespace() == ns!("")
        });
        Ok(())
    }

    fn do_set_attribute<F>(self,
                           local_name: Atom,
                           value: AttrValue,
                           name: Atom,
                           namespace: Namespace,
                           prefix: Option<DOMString>,
                           cb: F)
        where F: Fn(JSRef<Attr>) -> bool
    {
        let idx = self.attrs.borrow().iter()
                                     .map(|attr| attr.root())
                                     .position(|attr| cb(attr.r()));
        let (idx, set_type) = match idx {
            Some(idx) => (idx, AttrSettingType::ReplacedAttr),
            None => {
                let window = window_from_node(self).root();
                let attr = Attr::new(window.r(), local_name, value.clone(),
                                     name, namespace.clone(), prefix, Some(self));
                self.attrs.borrow_mut().push_unrooted(&attr);
                (self.attrs.borrow().len() - 1, AttrSettingType::FirstSetAttr)
            }
        };

        (*self.attrs.borrow())[idx].root().r().set_value(set_type, value, self);
    }

    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue {
        if *namespace == ns!("") {
            vtable_for(&NodeCast::from_ref(self))
                .parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value)
        }
    }

    fn remove_attribute(self, namespace: Namespace, name: &str) {
        let (_, local_name) = get_attribute_parts(name);
        let local_name = Atom::from_slice(local_name);

        let idx = self.attrs.borrow().iter().map(|attr| attr.root()).position(|attr| {
            *attr.r().local_name() == local_name
        });

        match idx {
            None => (),
            Some(idx) => {
                if namespace == ns!("") {
                    let attr = (*self.attrs.borrow())[idx].root();
                    vtable_for(&NodeCast::from_ref(self)).before_remove_attr(attr.r());
                }

                self.attrs.borrow_mut().remove(idx);

                let node: JSRef<Node> = NodeCast::from_ref(self);
                if node.is_in_doc() {
                    let document = document_from_node(self).root();
                    if local_name == atom!("style") {
                        document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                    } else {
                        document.r().content_changed(node, NodeDamage::OtherNodeDamage);
                    }
                }
            }
        };
    }

    fn has_class(self, name: &Atom) -> bool {
        let quirks_mode = {
            let node: JSRef<Node> = NodeCast::from_ref(self);
            let owner_doc = node.owner_doc().root();
            owner_doc.r().quirks_mode()
        };
        let is_equal = |&:lhs: &Atom, rhs: &Atom| match quirks_mode {
            NoQuirks | LimitedQuirks => lhs == rhs,
            Quirks => lhs.as_slice().eq_ignore_ascii_case(rhs.as_slice())
        };
        self.get_attribute(ns!(""), &atom!("class")).root().map(|attr| {
            attr.r().value().tokens().map(|tokens| {
                tokens.iter().any(|atom| is_equal(name, atom))
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn set_atomic_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice().eq_ignore_ascii_case(name.as_slice()));
        let value = AttrValue::from_atomic(value);
        self.set_attribute(name, value);
    }

    fn has_attribute(self, name: &Atom) -> bool {
        assert!(name.as_slice().bytes().all(|&:b| b.to_ascii_lowercase() == b));
        self.attrs.borrow().iter().map(|attr| attr.root()).any(|attr| {
            *attr.r().local_name() == *name && *attr.r().namespace() == ns!("")
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
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        if !self.has_attribute(name) {
            return "".to_owned();
        }
        let url = self.get_string_attribute(name);
        let doc = document_from_node(self).root();
        let base = doc.r().url();
        // https://html.spec.whatwg.org/multipage/infrastructure.html#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match UrlParser::new().base_url(&base).parse(url.as_slice()) {
            Ok(parsed) => parsed.serialize(),
            Err(_) => "".to_owned()
        }
    }
    fn set_url_attribute(self, name: &Atom, value: DOMString) {
        self.set_string_attribute(name, value);
    }

    fn get_string_attribute(self, name: &Atom) -> DOMString {
        match self.get_attribute(ns!(""), name) {
            Some(x) => x.root().r().Value(),
            None => "".to_owned()
        }
    }
    fn set_string_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        self.set_attribute(name, AttrValue::String(value));
    }

    fn get_tokenlist_attribute(self, name: &Atom) -> Vec<Atom> {
        self.get_attribute(ns!(""), name).root().map(|attr| {
            attr.r()
                .value()
                .tokens()
                .expect("Expected a TokenListAttrValue")
                .to_vec()
        }).unwrap_or(vec!())
    }

    fn set_tokenlist_attribute(self, name: &Atom, value: DOMString) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        self.set_attribute(name, AttrValue::from_serialized_tokenlist(value));
    }

    fn set_atomic_tokenlist_attribute(self, name: &Atom, tokens: Vec<Atom>) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        self.set_attribute(name, AttrValue::from_atomic_tokens(tokens));
    }

    fn get_uint_attribute(self, name: &Atom) -> u32 {
        assert!(name.as_slice().chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii_lowercase() == ch
        }));
        let attribute = self.get_attribute(ns!(""), name).root();
        match attribute {
            Some(attribute) => {
                match *attribute.r().value() {
                    AttrValue::UInt(_, value) => value,
                    _ => panic!("Expected an AttrValue::UInt: \
                                 implement parse_plain_attribute"),
                }
            }
            None => 0,
        }
    }
    fn set_uint_attribute(self, name: &Atom, value: u32) {
        assert!(name.as_slice() == name.as_slice().to_ascii_lowercase().as_slice());
        self.set_attribute(name, AttrValue::UInt(value.to_string(), value));
    }
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // http://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(self) -> Option<DOMString> {
        match self.namespace {
            ns!("") => None,
            Namespace(ref ns) => Some(ns.as_slice().to_owned())
        }
    }

    fn LocalName(self) -> DOMString {
        self.local_name.as_slice().to_owned()
    }

    // http://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // http://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(self) -> DOMString {
        let qualified_name = match self.prefix {
            Some(ref prefix) => {
                (format!("{}:{}",
                         prefix.as_slice(),
                         self.local_name.as_slice())).into_cow()
            },
            None => self.local_name.as_slice().into_cow()
        };
        if self.html_element_in_html_document() {
            qualified_name.as_slice().to_ascii_uppercase()
        } else {
            qualified_name.into_owned()
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
        self.class_list.or_init(|| DOMTokenList::new(self, &atom!("class")))
    }

    // http://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(self) -> Temporary<NamedNodeMap> {
        self.attr_list.or_init(|| {
            let doc = {
                let node: JSRef<Node> = NodeCast::from_ref(self);
                node.owner_doc().root()
            };
            let window = doc.r().window().root();
            NamedNodeMap::new(window.r(), self)
        })
    }

    // http://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(self, name: DOMString) -> Option<DOMString> {
        let name = self.parsed_name(name);
        self.get_attribute(ns!(""), &Atom::from_slice(name.as_slice())).root()
                     .map(|s| s.r().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = namespace::from_domstring(namespace);
        self.get_attribute(namespace, &Atom::from_slice(local_name.as_slice())).root()
                     .map(|attr| attr.r().Value())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(self,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        // Step 1.
        match xml_name_type(name.as_slice()) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Step 2.
        let name = self.parsed_name(name);

        // Step 3-5.
        let name = Atom::from_slice(name.as_slice());
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.do_set_attribute(name.clone(), value, name.clone(), ns!(""), None, |attr| {
            *attr.name() == name
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(self,
                      namespace_url: Option<DOMString>,
                      name: DOMString,
                      value: DOMString) -> ErrorResult {
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
        let xmlns = atom!("xmlns");

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
                              namespace.clone(), prefix.map(|s| s.to_owned()),
                              |attr| {
            *attr.local_name() == local_name &&
            *attr.namespace() == namespace
        });
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(self, name: DOMString) {
        let name = self.parsed_name(name);
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
        self.GetAttribute(name).is_some()
    }

    // http://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    fn GetElementsByTagName(self, localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name(window.r(), NodeCast::from_ref(self), localname)
    }

    fn GetElementsByTagNameNS(self, maybe_ns: Option<DOMString>,
                              localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name_ns(window.r(), NodeCast::from_ref(self), localname, maybe_ns)
    }

    fn GetElementsByClassName(self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_class_name(window.r(), NodeCast::from_ref(self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    fn GetClientRects(self) -> Temporary<DOMRectList> {
        let win = window_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rects = node.get_content_boxes();
        let rects: Vec<Root<DOMRect>> = rects.iter().map(|r| {
            DOMRect::new(
                win.r(),
                r.origin.y,
                r.origin.y + r.size.height,
                r.origin.x,
                r.origin.x + r.size.width).root()
        }).collect();

        DOMRectList::new(win.r(), rects.iter().map(|rect| rect.r()).collect())
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(self) -> Temporary<DOMRect> {
        let win = window_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        DOMRect::new(
            win.r(),
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
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
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
        match parse_author_origin_selector_list_from_str(selectors.as_slice()) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root: JSRef<Node> = NodeCast::from_ref(self);
                Ok(matches(selectors, &root, &mut None))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        match parse_author_origin_selector_list_from_str(selectors.as_slice()) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root: JSRef<Node> = NodeCast::from_ref(self);
                Ok(root.inclusive_ancestors()
                       .filter_map(ElementCast::to_ref)
                       .find(|element| matches(selectors, &NodeCast::from_ref(*element), &mut None))
                       .map(Temporary::from_rooted))
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
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
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
                // Modifying the `style` attribute might change style.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                let doc = document_from_node(*self).root();
                let base_url = doc.r().url();
                let value = attr.value();
                let style = Some(parse_style_attribute(value.as_slice(), &base_url));
                *self.style_attribute.borrow_mut() = style;

                if node.is_in_doc() {
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("class") => {
                // Modifying a class can change style.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("id") => {
                // Modifying an ID might change style.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                let value = attr.value();
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    if !value.as_slice().is_empty() {
                        let value = Atom::from_slice(value.as_slice());
                        doc.r().register_named_element(*self, value);
                    }
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            _ => {
                // Modifying any other attribute might change arbitrary things.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::OtherNodeDamage);
                }
            }
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("style") => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() = None;

                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("id") => {
                // Modifying an ID can change style.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                let value = attr.value();
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    if !value.as_slice().is_empty() {
                        let value = Atom::from_slice(value.as_slice());
                        doc.r().unregister_named_element(*self, value);
                    }
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("class") => {
                // Modifying a class can change style.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            _ => {
                // Modifying any other attribute might change arbitrary things.
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    doc.r().content_changed(node, NodeDamage::OtherNodeDamage);
                }
            }
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("id") => AttrValue::from_atomic(value),
            &atom!("class") => AttrValue::from_serialized_tokenlist(value),
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
                let value = attr.r().Value();
                if !value.is_empty() {
                    let value = Atom::from_slice(value.as_slice());
                    doc.r().register_named_element(*self, value);
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
                let value = attr.r().Value();
                if !value.is_empty() {
                    let value = Atom::from_slice(value.as_slice());
                    doc.r().unregister_named_element(*self, value);
                }
            }
            _ => ()
        }
    }
}

impl<'a> style::node::TElement<'a> for JSRef<'a, Element> {
    #[allow(unsafe_blocks)]
    fn get_attr(self, namespace: &Namespace, attr: &Atom) -> Option<&'a str> {
        self.get_attribute(namespace.clone(), attr).root().map(|attr| {
            // This transmute is used to cheat the lifetime restriction.
            unsafe { mem::transmute(attr.r().value().as_slice()) }
        })
    }
    #[allow(unsafe_blocks)]
    fn get_attrs(self, attr: &Atom) -> Vec<&'a str> {
        self.get_attributes(attr).into_iter().map(|attr| attr.root()).map(|attr| {
            // This transmute is used to cheat the lifetime restriction.
            unsafe { mem::transmute(attr.r().value().as_slice()) }
        }).collect()
    }
    fn get_link(self) -> Option<&'a str> {
        // FIXME: This is HTML only.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => self.get_attr(&ns!(""), &atom!("href")),
            _ => None,
         }
    }
    fn get_local_name(self) -> &'a Atom {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn get_local_name<'a, T: ElementHelpers<'a>>(this: T) -> &'a Atom {
            this.local_name()
        }

        get_local_name(self)
    }
    fn get_namespace(self) -> &'a Namespace {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn get_namespace<'a, T: ElementHelpers<'a>>(this: T) -> &'a Namespace {
            this.namespace()
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
            match *attr.r().value() {
                AttrValue::Atom(ref val) => val.clone(),
                _ => panic!("`id` attribute should be AttrValue::Atom"),
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
    fn get_checked_state(self) -> bool {
        let input_element: Option<JSRef<HTMLInputElement>> = HTMLInputElementCast::to_ref(self);
        match input_element {
            Some(input) => input.Checked(),
            None => false,
        }
    }
    fn get_indeterminate_state(self) -> bool {
        let input_element: Option<JSRef<HTMLInputElement>> = HTMLInputElementCast::to_ref(self);
        match input_element {
            Some(input) => input.get_indeterminate_state(),
            None => false,
        }
    }
    fn has_class(self, name: &Atom) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn has_class<T: AttributeHandlers>(this: T, name: &Atom) -> bool {
            this.has_class(name)
        }

        has_class(self, name)
    }
    fn each_class<F>(self, mut callback: F)
        where F: FnMut(&Atom)
    {
        match self.get_attribute(ns!(""), &atom!("class")).root() {
            None => {}
            Some(ref attr) => {
                match attr.r().value().tokens() {
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
    fn has_nonzero_border(self) -> bool {
        let table_element: Option<JSRef<HTMLTableElement>> = HTMLTableElementCast::to_ref(self);
        match table_element {
            None => false,
            Some(this) => {
                match this.get_border() {
                    None | Some(0) => false,
                    Some(_) => true,
                }
            }
        }
    }
}

pub trait ActivationElementHelpers<'a> {
    fn as_maybe_activatable(&'a self) -> Option<&'a (Activatable + 'a)>;
    fn click_in_progress(self) -> bool;
    fn set_click_in_progress(self, click: bool);
    fn nearest_activable_element(self) -> Option<Temporary<Element>>;
    fn authentic_click_activation<'b>(self, event: JSRef<'b, Event>);
}

impl<'a> ActivationElementHelpers<'a> for JSRef<'a, Element> {
    fn as_maybe_activatable(&'a self) -> Option<&'a (Activatable + 'a)> {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element: &'a JSRef<'a, HTMLInputElement> = HTMLInputElementCast::to_borrowed_ref(self).unwrap();
                if element.is_instance_activatable() {
                    Some(element as &'a (Activatable + 'a))
                } else {
                    None
                }
            },
            _ => {
                None
            }
        }
    }

    fn click_in_progress(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_flag(CLICK_IN_PROGRESS)
    }

    fn set_click_in_progress(self, click: bool) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.set_flag(CLICK_IN_PROGRESS, click)
    }

    // https://html.spec.whatwg.org/multipage/interaction.html#nearest-activatable-element
    fn nearest_activable_element(self) -> Option<Temporary<Element>> {
        match self.as_maybe_activatable() {
            Some(el) => Some(Temporary::from_rooted(el.as_element().root().r())),
            None => {
                let node: JSRef<Node> = NodeCast::from_ref(self);
                node.ancestors()
                    .filter_map(|node| {
                        let e: Option<JSRef<Element>> = ElementCast::to_ref(node);
                        e
                    })
                    .filter(|e| e.as_maybe_activatable().is_some()).next()
                    .map(|r| Temporary::from_rooted(r))
            }
        }
    }

    /// Please call this method *only* for real click events
    ///
    /// https://html.spec.whatwg.org/multipage/interaction.html#run-authentic-click-activation-steps
    ///
    /// Use an element's synthetic click activation (or handle_event) for any script-triggered clicks.
    /// If the spec says otherwise, check with Manishearth first
    fn authentic_click_activation<'b>(self, event: JSRef<'b, Event>) {
        // Not explicitly part of the spec, however this helps enforce the invariants
        // required to save state between pre-activation and post-activation
        // since we cannot nest authentic clicks (unlike synthetic click activation, where
        // the script can generate more click events from the handler)
        assert!(!self.click_in_progress());

        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        // Step 2 (requires canvas support)
        // Step 3
        self.set_click_in_progress(true);
        // Step 4
        let e = self.nearest_activable_element().root();
        match e {
            Some(el) => match el.r().as_maybe_activatable() {
                Some(elem) => {
                    // Step 5-6
                    elem.pre_click_activation();
                    event.fire(target);
                    if !event.DefaultPrevented() {
                        // post click activation
                        elem.activation_behavior();
                    } else {
                        elem.canceled_activation();
                    }
                }
                // Step 6
                None => {event.fire(target);}
            },
            // Step 6
            None => {event.fire(target);}
        }
        // Step 7
        self.set_click_in_progress(false);
    }
}
