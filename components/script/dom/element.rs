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
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, ElementDerived, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLInputElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLTableElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, HTMLTableCellElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableRowElementDerived, HTMLTextAreaElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableSectionElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::HTMLTableDataCellElementDerived;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{InvalidCharacter, Syntax};
use dom::bindings::error::Error::NoModificationAllowed;
use dom::bindings::js::{JS, JSRef, LayoutJS, MutNullableHeap};
use dom::bindings::js::{OptionalRootable, Rootable, RootedReference};
use dom::bindings::js::{Temporary, TemporaryPushable};
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::{xml_name_type, validate_and_extract};
use dom::bindings::utils::XMLName::InvalidXMLName;
use dom::create::create_element;
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::document::{Document, DocumentHelpers, LayoutDocumentHelpers};
use dom::domtokenlist::DOMTokenList;
use dom::event::{Event, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementHelpers};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElementTypeId;
use dom::htmlinputelement::{HTMLInputElement, RawLayoutHTMLInputElementHelpers, HTMLInputElementHelpers};
use dom::htmltableelement::{HTMLTableElement, HTMLTableElementHelpers};
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementHelpers};
use dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementHelpers};
use dom::htmltablesectionelement::{HTMLTableSectionElement, HTMLTableSectionElementHelpers};
use dom::htmltextareaelement::{HTMLTextAreaElement, RawLayoutHTMLTextAreaElementHelpers};
use dom::node::{CLICK_IN_PROGRESS, LayoutNodeHelpers, Node, NodeHelpers, NodeTypeId};
use dom::node::{document_from_node, NodeDamage};
use dom::node::{window_from_node};
use dom::nodelist::NodeList;
use dom::virtualmethods::{VirtualMethods, vtable_for};

use devtools_traits::AttrInfo;
use style;
use style::legacy::{UnsignedIntegerAttribute, IntegerAttribute, LengthAttribute, from_declaration};
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration, parse_style_attribute};
use style::properties::DeclaredValue::SpecifiedValue;
use style::values::specified::CSSColor;
use util::namespace;
use util::smallvec::VecLike;
use util::str::{DOMString, LengthOrPercentageOrAuto};

use cssparser::Color;
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{IncludeNode, ChildrenOnly};
use html5ever::tree_builder::{NoQuirks, LimitedQuirks, Quirks};
use selectors::matching::{matches, DeclarationBlock};
use selectors::parser::parse_author_origin_selector_list_from_str;
use string_cache::{Atom, Namespace, QualName};
use url::UrlParser;

use std::ascii::AsciiExt;
use std::borrow::{Cow, ToOwned};
use std::cell::{Ref, RefMut};
use std::default::Default;
use std::mem;
use std::sync::Arc;

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: Atom,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    style_attribute: DOMRefCell<Option<PropertyDeclarationBlock>>,
    attr_list: MutNullableHeap<JS<NamedNodeMap>>,
    class_list: MutNullableHeap<JS<DOMTokenList>>,
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

#[derive(Copy, Clone, PartialEq, Debug)]
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
    pub fn create(name: QualName, prefix: Option<Atom>,
                  document: JSRef<Document>, creator: ElementCreator)
                  -> Temporary<Element> {
        create_element(name, prefix, document, creator)
    }

    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: JSRef<Document>) -> Element {
        Element {
            node: Node::new_inherited(NodeTypeId::Element(type_id), document),
            local_name: Atom::from_slice(&local_name),
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

#[allow(unsafe_code)]
pub trait RawLayoutElementHelpers {
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                      -> Option<&'a str>;
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str>;
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom) -> Option<Atom>;
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool;
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]>;

    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    unsafe fn get_length_attribute_for_layout(&self, length_attribute: LengthAttribute)
                                              -> LengthOrPercentageOrAuto;
    unsafe fn get_integer_attribute_for_layout(&self, integer_attribute: IntegerAttribute)
                                               -> Option<i32>;
    unsafe fn get_checked_state_for_layout(&self) -> bool;
    unsafe fn get_indeterminate_state_for_layout(&self) -> bool;
    unsafe fn get_unsigned_integer_attribute_for_layout(&self, attribute: UnsignedIntegerAttribute)
                                                        -> Option<u32>;

    fn local_name<'a>(&'a self) -> &'a Atom;
    fn namespace<'a>(&'a self) -> &'a Namespace;
    fn style_attribute<'a>(&'a self) -> &'a DOMRefCell<Option<PropertyDeclarationBlock>>;
}

#[inline]
#[allow(unsafe_code)]
unsafe fn get_attr_for_layout(elem: &Element, namespace: &Namespace, name: &Atom) -> Option<LayoutJS<Attr>> {
    // cast to point to T in RefCell<T> directly
    let attrs = elem.attrs.borrow_for_layout();
    attrs.iter().find(|attr: & &JS<Attr>| {
        let attr = attr.to_layout().unsafe_get();
        *name == (*attr).local_name_atom_forever() &&
        (*attr).namespace() == namespace
    }).map(|attr| attr.to_layout())
}

#[allow(unsafe_code)]
impl RawLayoutElementHelpers for Element {
    #[inline]
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                          -> Option<&'a str> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            (*attr.unsafe_get()).value_ref_forever()
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
        get_attr_for_layout(self, namespace, name).and_then(|attr| {
            (*attr.unsafe_get()).value_atom_forever()
        })
    }

    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        get_attr_for_layout(self, &ns!(""), &atom!("class")).map_or(false, |attr| {
            (*attr.unsafe_get()).value_tokens_forever().unwrap().iter().any(|atom| atom == name)
        })
    }

    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        get_attr_for_layout(self, &ns!(""), &atom!("class")).map(|attr| {
            (*attr.unsafe_get()).value_tokens_forever().unwrap()
        })
    }

    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        let bgcolor = if self.is_htmlbodyelement() {
            let this: &HTMLBodyElement = mem::transmute(self);
            this.get_background_color()
        } else if self.is_htmltableelement() {
            let this: &HTMLTableElement = mem::transmute(self);
            this.get_background_color()
        } else if self.is_htmltabledatacellelement() {
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
        };

        if let Some(color) = bgcolor {
            hints.push(from_declaration(
                PropertyDeclaration::BackgroundColor(SpecifiedValue(
                    CSSColor { parsed: Color::RGBA(color), authored: None }))));
        }
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
            UnsignedIntegerAttribute::CellSpacing => {
                if self.is_htmltableelement() {
                    let this: &HTMLTableElement = mem::transmute(self);
                    this.get_cellspacing()
                } else {
                    // Don't panic since `display` can cause this to be called on arbitrary
                    // elements.
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
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn has_attr_for_layout(&self, namespace: &Namespace, name: &Atom) -> bool;
}

impl LayoutElementHelpers for LayoutJS<Element> {
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if (*self.unsafe_get()).namespace != ns!(HTML) {
            return false
        }
        let node = NodeCast::from_layout_js(&self);
        node.owner_doc_for_layout().is_html_document_for_layout()
    }

    #[allow(unsafe_code)]
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
    fn serialize(self, traversal_scope: TraversalScope) -> Fallible<DOMString>;
    fn get_root_element(self) -> Option<Temporary<Element>>;
    fn lookup_prefix(self, namespace: Option<DOMString>) -> Option<DOMString>;
}

impl<'a> ElementHelpers<'a> for JSRef<'a, Element> {
    fn html_element_in_html_document(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        self.namespace == ns!(HTML) && node.is_in_html_doc()
    }

    fn local_name(self) -> &'a Atom {
        &self.extended_deref().local_name
    }

    fn parsed_name(self, name: DOMString) -> DOMString {
        if self.html_element_in_html_document() {
            name.to_ascii_lowercase()
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
        let mut summarized = vec!();
        for i in 0..attrs.r().Length() {
            let attr = attrs.r().Item(i).unwrap().root();
            summarized.push(attr.r().summarize());
        }
        summarized
    }

    fn is_void(self) -> bool {
        if self.namespace != ns!(HTML) {
            return false
        }
        match &*self.local_name {
            /* List of void elements from
            https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm */
            "area" | "base" | "basefont" | "bgsound" | "br" | "col" | "embed" |
            "frame" | "hr" | "img" | "input" | "keygen" | "link" | "menuitem" |
            "meta" | "param" | "source" | "track" | "wbr" => true,
            _ => false
        }
    }

    fn remove_inline_style_property(self, property: DOMString) {
        let mut inline_declarations = self.style_attribute.borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let index = declarations.normal
                                    .iter()
                                    .position(|decl| decl.name() == property);
            if let Some(index) = index {
                declarations.normal.make_unique().remove(index);
                return;
            }

            let index = declarations.important
                                    .iter()
                                    .position(|decl| decl.name() == property);
            if let Some(index) = index {
                declarations.important.make_unique().remove(index);
                return;
            }
        }
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
                        .find(|decl| decl.matches(&property))
                        .map(|decl| decl.clone())
        })
    }

    fn get_important_inline_style_declaration(self, property: &Atom) -> Option<PropertyDeclaration> {
        let inline_declarations = self.style_attribute.borrow();
        inline_declarations.as_ref().and_then(|declarations| {
            declarations.important
                        .iter()
                        .find(|decl| decl.matches(&property))
                        .map(|decl| decl.clone())
        })
    }

    fn serialize(self, traversal_scope: TraversalScope) -> Fallible<DOMString> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let mut writer = vec![];
        match serialize(&mut writer, &node,
                        SerializeOpts {
                            traversal_scope: traversal_scope,
                            .. Default::default()
                        }) {
            Ok(()) => Ok(String::from_utf8(writer).unwrap()),
            Err(_) => panic!("Cannot serialize element"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#root-element
    fn get_root_element(self) -> Option<Temporary<Element>> {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.ancestors().last().map(ElementCast::to_temporary) {
            Some(n) => n,
            None => Some(self).map(Temporary::from_rooted),
        }
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace-prefix
    fn lookup_prefix(self, namespace: Option<DOMString>) -> Option<DOMString> {
        for node in NodeCast::from_ref(self).inclusive_ancestors() {
            match ElementCast::to_ref(node.root().r()) {
                Some(element) => {
                    // Step 1.
                    if element.GetNamespaceURI() == namespace && element.GetPrefix().is_some() {
                        return element.GetPrefix();
                    }

                    // Step 2.
                    let attrs = element.Attributes().root();
                    for i in 0..attrs.r().Length() {
                        let attr = attrs.r().Item(i).unwrap().root();
                        if attr.r().GetPrefix() == Some("xmlns".to_owned()) &&
                           Some(attr.r().Value()) == namespace {
                            return Some(attr.r().LocalName());
                        }
                    }
                },
                None => return None,
            }
        }
        None
    }
}

pub trait FocusElementHelpers {
    /// https://html.spec.whatwg.org/multipage/#focusable-area
    fn is_focusable_area(self) -> bool;

    /// https://html.spec.whatwg.org/multipage/#concept-element-disabled
    fn is_actually_disabled(self) -> bool;
}

impl<'a> FocusElementHelpers for JSRef<'a, Element> {
    fn is_focusable_area(self) -> bool {
        if self.is_actually_disabled() {
            return false;
        }
        // TODO: Check whether the element is being rendered (i.e. not hidden).
        // TODO: Check the tabindex focus flag.
        // https://html.spec.whatwg.org/multipage/#specially-focusable
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                true
            }
            _ => false
        }
    }

    fn is_actually_disabled(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
                node.get_disabled_state()
            }
            // TODO:
            // an optgroup element that has a disabled attribute
            // a menuitem element that has a disabled attribute
            // a fieldset element that is a disabled fieldset
            _ => false
        }
    }
}

pub trait AttributeHandlers {
    /// Returns the attribute with given namespace and case-sensitive local
    /// name, if any.
    fn get_attribute(self, namespace: &Namespace, local_name: &Atom)
                     -> Option<Temporary<Attr>>;
    /// Returns the first attribute with any namespace and given case-sensitive
    /// name, if any.
    fn get_attribute_by_name(self, name: DOMString) -> Option<Temporary<Attr>>;
    fn get_attributes(self, local_name: &Atom, attributes: &mut RootedVec<JS<Attr>>);
    fn set_attribute_from_parser(self,
                                 name: QualName,
                                 value: DOMString,
                                 prefix: Option<Atom>);
    fn set_attribute(self, name: &Atom, value: AttrValue);
    fn set_custom_attribute(self, name: DOMString, value: DOMString) -> ErrorResult;
    fn do_set_attribute<F>(self, local_name: Atom, value: AttrValue,
                           name: Atom, namespace: Namespace,
                           prefix: Option<Atom>, cb: F)
        where F: Fn(JSRef<Attr>) -> bool;
    fn parse_attribute(self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue;

    /// Removes the first attribute with any given namespace and case-sensitive local
    /// name, if any.
    fn remove_attribute(self, namespace: &Namespace, local_name: &Atom)
                        -> Option<Temporary<Attr>>;
    /// Removes the first attribute with any namespace and given case-sensitive name.
    fn remove_attribute_by_name(self, name: &Atom) -> Option<Temporary<Attr>>;
    /// Removes the first attribute that satisfies `find`.
    fn do_remove_attribute<F>(self, find: F) -> Option<Temporary<Attr>>
        where F: Fn(JSRef<Attr>) -> bool;

    fn has_class(self, name: &Atom) -> bool;

    fn set_atomic_attribute(self, local_name: &Atom, value: DOMString);

    // https://www.whatwg.org/html/#reflecting-content-attributes-in-idl-attributes
    fn has_attribute(self, local_name: &Atom) -> bool;
    fn set_bool_attribute(self, local_name: &Atom, value: bool);
    fn get_url_attribute(self, local_name: &Atom) -> DOMString;
    fn set_url_attribute(self, local_name: &Atom, value: DOMString);
    fn get_string_attribute(self, local_name: &Atom) -> DOMString;
    fn set_string_attribute(self, local_name: &Atom, value: DOMString);
    fn get_tokenlist_attribute(self, local_name: &Atom) -> Vec<Atom>;
    fn set_tokenlist_attribute(self, local_name: &Atom, value: DOMString);
    fn set_atomic_tokenlist_attribute(self, local_name: &Atom, tokens: Vec<Atom>);
    fn get_uint_attribute(self, local_name: &Atom) -> u32;
    fn set_uint_attribute(self, local_name: &Atom, value: u32);
}

impl<'a> AttributeHandlers for JSRef<'a, Element> {
    fn get_attribute(self, namespace: &Namespace, local_name: &Atom) -> Option<Temporary<Attr>> {
        let mut attributes = RootedVec::new();
        self.get_attributes(local_name, &mut attributes);
        attributes.iter().map(|attr| attr.root()).find(|attr| attr.r().namespace() == namespace).map(|x| Temporary::from_rooted(x.r()))
    }

    // https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name
    fn get_attribute_by_name(self, name: DOMString) -> Option<Temporary<Attr>> {
        let name = &Atom::from_slice(&self.parsed_name(name));
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let attrs = self.attrs.borrow();
        attrs.iter().map(|attr| attr.root())
             .find(|a| a.r().name() == name)
             .map(|x| Temporary::from_rooted(x.r()))
    }

    // https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name
    fn get_attributes(self, local_name: &Atom, attributes: &mut RootedVec<JS<Attr>>) {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let attrs = self.attrs.borrow();
        for ref attr in attrs.iter().map(|attr| attr.root()) {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let attr_local_name = attr.local_name();
            if attr_local_name == local_name {
                attributes.push(JS::from_rooted(attr));
            }
        }
    }

    fn set_attribute_from_parser(self,
                                 qname: QualName,
                                 value: DOMString,
                                 prefix: Option<Atom>) {
        // Don't set if the attribute already exists, so we can handle add_attrs_if_missing
        if self.attrs.borrow().iter().map(|attr| attr.root())
                .any(|a| *a.r().local_name() == qname.local && *a.r().namespace() == qname.ns) {
            return;
        }

        let name = match prefix {
            None => qname.local.clone(),
            Some(ref prefix) => {
                let name = format!("{}:{}", &**prefix, &*qname.local);
                Atom::from_slice(&name)
            },
        };
        let value = self.parse_attribute(&qname.ns, &qname.local, value);
        self.do_set_attribute(qname.local, value, name, qname.ns, prefix, |_| false)
    }

    fn set_attribute(self, name: &Atom, value: AttrValue) {
        assert!(&**name == name.to_ascii_lowercase());
        assert!(!name.contains(":"));

        self.do_set_attribute(name.clone(), value, name.clone(),
            ns!(""), None, |attr| attr.local_name() == name);
    }

    // https://html.spec.whatwg.org/multipage/#attr-data-*
    fn set_custom_attribute(self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        match xml_name_type(&name) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Steps 2-5.
        let name = Atom::from_slice(&name);
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
                           prefix: Option<Atom>,
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

    fn remove_attribute(self, namespace: &Namespace, local_name: &Atom)
                        -> Option<Temporary<Attr>> {
        self.do_remove_attribute(|attr| {
            attr.namespace() == namespace && attr.local_name() == local_name
        })
    }

    fn remove_attribute_by_name(self, name: &Atom) -> Option<Temporary<Attr>> {
        self.do_remove_attribute(|attr| attr.name() == name)
    }

    fn do_remove_attribute<F>(self, find: F) -> Option<Temporary<Attr>>
        where F: Fn(JSRef<Attr>) -> bool
    {
        let idx = self.attrs.borrow().iter()
                                     .map(|attr| attr.root())
                                     .position(|attr| find(attr.r()));

        idx.map(|idx| {
            let attr = (*self.attrs.borrow())[idx].root();
            if attr.r().namespace() == &ns!("") {
                vtable_for(&NodeCast::from_ref(self)).before_remove_attr(attr.r());
            }

            self.attrs.borrow_mut().remove(idx);
            attr.r().set_owner(None);

            let node: JSRef<Node> = NodeCast::from_ref(self);
            if node.is_in_doc() {
                let document = document_from_node(self).root();
                let damage = if attr.r().local_name() == &atom!("style") {
                    NodeDamage::NodeStyleDamaged
                } else {
                    NodeDamage::OtherNodeDamage
                };
                document.r().content_changed(node, damage);
            }
            Temporary::from_rooted(attr.r())
        })
    }

    fn has_class(self, name: &Atom) -> bool {
        let quirks_mode = {
            let node: JSRef<Node> = NodeCast::from_ref(self);
            let owner_doc = node.owner_doc().root();
            owner_doc.r().quirks_mode()
        };
        let is_equal = |lhs: &Atom, rhs: &Atom| match quirks_mode {
            NoQuirks | LimitedQuirks => lhs == rhs,
            Quirks => lhs.eq_ignore_ascii_case(&rhs)
        };
        self.get_attribute(&ns!(""), &atom!("class")).root().map(|attr| {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let value = attr.value();
            value.tokens().map(|tokens| {
                tokens.iter().any(|atom| is_equal(name, atom))
            }).unwrap_or(false)
        }).unwrap_or(false)
    }

    fn set_atomic_attribute(self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value);
        self.set_attribute(local_name, value);
    }

    fn has_attribute(self, local_name: &Atom) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let attrs = self.attrs.borrow();
        attrs.iter().map(|attr| attr.root()).any(|attr| {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            attr.local_name() == local_name && attr.namespace() == &ns!("")
        })
    }

    fn set_bool_attribute(self, local_name: &Atom, value: bool) {
        if self.has_attribute(local_name) == value { return; }
        if value {
            self.set_string_attribute(local_name, String::new());
        } else {
            self.remove_attribute(&ns!(""), local_name);
        }
    }

    fn get_url_attribute(self, local_name: &Atom) -> DOMString {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        if !self.has_attribute(local_name) {
            return "".to_owned();
        }
        let url = self.get_string_attribute(local_name);
        let doc = document_from_node(self).root();
        let base = doc.r().url();
        // https://html.spec.whatwg.org/multipage/#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match UrlParser::new().base_url(&base).parse(&url) {
            Ok(parsed) => parsed.serialize(),
            Err(_) => "".to_owned()
        }
    }
    fn set_url_attribute(self, local_name: &Atom, value: DOMString) {
        self.set_string_attribute(local_name, value);
    }

    fn get_string_attribute(self, local_name: &Atom) -> DOMString {
        match self.get_attribute(&ns!(""), local_name) {
            Some(x) => x.root().r().Value(),
            None => "".to_owned()
        }
    }
    fn set_string_attribute(self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value));
    }

    fn get_tokenlist_attribute(self, local_name: &Atom) -> Vec<Atom> {
        self.get_attribute(&ns!(""), local_name).root().map(|attr| {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let value = attr.value();
            value.tokens()
                 .expect("Expected a TokenListAttrValue")
                 .to_vec()
        }).unwrap_or(vec!())
    }

    fn set_tokenlist_attribute(self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_serialized_tokenlist(value));
    }

    fn set_atomic_tokenlist_attribute(self, local_name: &Atom, tokens: Vec<Atom>) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens));
    }

    fn get_uint_attribute(self, local_name: &Atom) -> u32 {
        assert!(local_name.chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii_lowercase() == ch
        }));
        let attribute = self.get_attribute(&ns!(""), local_name).root();
        match attribute {
            Some(ref attribute) => {
                match *attribute.r().value() {
                    AttrValue::UInt(_, value) => value,
                    _ => panic!("Expected an AttrValue::UInt: \
                                 implement parse_plain_attribute"),
                }
            }
            None => 0,
        }
    }
    fn set_uint_attribute(self, local_name: &Atom, value: u32) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::UInt(value.to_string(), value));
    }
}

impl<'a> ElementMethods for JSRef<'a, Element> {
    // https://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(self) -> Option<DOMString> {
        match self.namespace {
            ns!("") => None,
            Namespace(ref ns) => Some((**ns).to_owned())
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-localname
    fn LocalName(self) -> DOMString {
        (*self.local_name).to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // https://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(self) -> DOMString {
        let qualified_name = match self.prefix {
            Some(ref prefix) => {
                Cow::Owned(format!("{}:{}", &**prefix, &*self.local_name))
            },
            None => Cow::Borrowed(&*self.local_name)
        };
        if self.html_element_in_html_document() {
            qualified_name.to_ascii_uppercase()
        } else {
            qualified_name.into_owned()
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-id
    fn Id(self) -> DOMString {
        self.get_string_attribute(&atom!("id"))
    }

    // https://dom.spec.whatwg.org/#dom-element-id
    fn SetId(self, id: DOMString) {
        self.set_atomic_attribute(&atom!("id"), id);
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(self) -> DOMString {
        self.get_string_attribute(&atom!("class"))
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(self, class: DOMString) {
        self.set_tokenlist_attribute(&atom!("class"), class);
    }

    // https://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(self) -> Temporary<DOMTokenList> {
        self.class_list.or_init(|| DOMTokenList::new(self, &atom!("class")))
    }

    // https://dom.spec.whatwg.org/#dom-element-attributes
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

    // https://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(self, name: DOMString) -> Option<DOMString> {
        self.get_attribute_by_name(name).root()
                     .map(|s| s.r().Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = &namespace::from_domstring(namespace);
        self.get_attribute(namespace, &Atom::from_slice(&local_name)).root()
                     .map(|attr| attr.r().Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(self,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        // Step 1.
        if xml_name_type(&name) == InvalidXMLName {
            return Err(InvalidCharacter);
        }

        // Step 2.
        let name = self.parsed_name(name);

        // Step 3-5.
        let name = Atom::from_slice(&name);
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.do_set_attribute(name.clone(), value, name.clone(), ns!(""), None, |attr| {
            *attr.name() == name
        });
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(self,
                      namespace: Option<DOMString>,
                      qualified_name: DOMString,
                      value: DOMString) -> ErrorResult {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let qualified_name = Atom::from_slice(&qualified_name);
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.do_set_attribute(local_name.clone(), value, qualified_name,
                              namespace.clone(), prefix, |attr| {
            *attr.local_name() == local_name &&
            *attr.namespace() == namespace
        });
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(self, name: DOMString) {
        let name = Atom::from_slice(&self.parsed_name(name));
        self.remove_attribute_by_name(&name);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(self,
                         namespace: Option<DOMString>,
                         local_name: DOMString) {
        let namespace = namespace::from_domstring(namespace);
        let local_name = Atom::from_slice(&local_name);
        self.remove_attribute(&namespace, &local_name);
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    fn GetElementsByTagName(self, localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name(window.r(), NodeCast::from_ref(self), localname)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(self, maybe_ns: Option<DOMString>,
                              localname: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_tag_name_ns(window.r(), NodeCast::from_ref(self), localname, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(self, classes: DOMString) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::by_class_name(window.r(), NodeCast::from_ref(self), classes)
    }

    // http://dev.w3.org/csswg/cssom-view/#dom-element-getclientrects
    fn GetClientRects(self) -> Temporary<DOMRectList> {
        let win = window_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let raw_rects = node.get_content_boxes();
        let mut rects = RootedVec::new();
        for rect in raw_rects.iter() {
            let rect = DOMRect::new(win.r(),
                                    rect.origin.y,
                                    rect.origin.y + rect.size.height,
                                    rect.origin.x,
                                    rect.origin.x + rect.size.width);
            rects.push(JS::from_rooted(rect));
        }

        DOMRectList::new(win.r(), &rects)
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

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-innerHTML
    fn GetInnerHTML(self) -> Fallible<DOMString> {
        //XXX TODO: XML case
        self.serialize(ChildrenOnly)
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-innerHTML
    fn SetInnerHTML(self, value: DOMString) -> Fallible<()> {
        let context_node: JSRef<Node> = NodeCast::from_ref(self);
        // Step 1.
        let frag = try!(context_node.parse_fragment(value));
        // Step 2.
        Node::replace_all(Some(NodeCast::from_ref(frag.root().r())), context_node);
        Ok(())
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn GetOuterHTML(self) -> Fallible<DOMString> {
        self.serialize(IncludeNode)
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn SetOuterHTML(self, value: DOMString) -> Fallible<()> {
        let context_document = document_from_node(self).root();
        let context_node: JSRef<Node> = NodeCast::from_ref(self);
        // Step 1.
        let context_parent = match context_node.parent_node() {
            // Step 2.
            None => return Ok(()),
            Some(parent) => parent.root()
        };

        let parent = match context_parent.r().type_id() {
            // Step 3.
            NodeTypeId::Document => return Err(NoModificationAllowed),

            // Step 4.
            NodeTypeId::DocumentFragment => {
                let body_elem = Element::create(QualName::new(ns!(HTML), atom!(body)),
                                                None, context_document.r(),
                                                ElementCreator::ScriptCreated);
                let body_node: Temporary<Node> = NodeCast::from_temporary(body_elem);
                body_node.root()
            },
            _ => context_node.parent_node().unwrap().root()
        };

        // Step 5.
        let frag = try!(parent.r().parse_fragment(value));
        // Step 6.
        try!(context_parent.r().ReplaceChild(NodeCast::from_ref(frag.root().r()),
                                             context_node));
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).preceding_siblings()
                                .filter_map(ElementCast::to_temporary).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).following_siblings()
                                .filter_map(ElementCast::to_temporary).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(self) -> Temporary<HTMLCollection> {
        let window = window_from_node(self).root();
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(self) -> Option<Temporary<Element>> {
        NodeCast::from_ref(self).rev_children().filter_map(ElementCast::to_temporary).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(self) -> u32 {
        NodeCast::from_ref(self).child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        let root: JSRef<Node> = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(self) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-element-matches
    fn Matches(self, selectors: DOMString) -> Fallible<bool> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root: JSRef<Node> = NodeCast::from_ref(self);
                Ok(matches(selectors, &root, &mut None))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root: JSRef<Node> = NodeCast::from_ref(self);
                for element in root.inclusive_ancestors() {
                    let element = element.root();
                    if let Some(element) = ElementCast::to_ref(element.r()) {
                        if matches(selectors, &NodeCast::from_ref(element), &mut None) {
                            return Ok(Some(Temporary::from_rooted(element)));
                        }
                    }
                }
                Ok(None)
            }
        }
    }
}

impl<'a> VirtualMethods for JSRef<'a, Element> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let node: &JSRef<Node> = NodeCast::from_borrowed_ref(self);
        Some(node as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match attr.local_name() {
            &atom!("style") => {
                // Modifying the `style` attribute might change style.
                let doc = document_from_node(*self).root();
                let base_url = doc.r().url();
                let value = attr.value();
                let style = Some(parse_style_attribute(&value, &base_url));
                *self.style_attribute.borrow_mut() = style;

                if node.is_in_doc() {
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("class") => {
                // Modifying a class can change style.
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("id") => {
                // Modifying an ID might change style.
                let value = attr.value();
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    if !value.is_empty() {
                        let value = value.atom().unwrap().clone();
                        doc.r().register_named_element(*self, value);
                    }
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            _ => {
                // Modifying any other attribute might change arbitrary things.
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::OtherNodeDamage);
                }
            }
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match attr.local_name() {
            &atom!("style") => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() = None;

                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("id") => {
                // Modifying an ID can change style.
                let value = attr.value();
                if node.is_in_doc() {
                    let doc = document_from_node(*self).root();
                    if !value.is_empty() {
                        let value = value.atom().unwrap().clone();
                        doc.r().unregister_named_element(*self, value);
                    }
                    doc.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            &atom!("class") => {
                // Modifying a class can change style.
                if node.is_in_doc() {
                    let document = document_from_node(*self).root();
                    document.r().content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            }
            _ => {
                // Modifying any other attribute might change arbitrary things.
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
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if !tree_in_doc { return; }

        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("id")).root() {
            let doc = document_from_node(*self).root();
            let value = attr.r().Value();
            if !value.is_empty() {
                let value = Atom::from_slice(&value);
                doc.r().register_named_element(*self, value);
            }
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        if !tree_in_doc { return; }

        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("id")).root() {
            let doc = document_from_node(*self).root();
            let value = attr.r().Value();
            if !value.is_empty() {
                let value = Atom::from_slice(&value);
                doc.r().unregister_named_element(*self, value);
            }
        }
    }
}

impl<'a> style::node::TElement<'a> for JSRef<'a, Element> {
    #[allow(unsafe_code)]
    fn get_attr(self, namespace: &Namespace, local_name: &Atom) -> Option<&'a str> {
        self.get_attribute(namespace, local_name).root().map(|attr| {
            // This transmute is used to cheat the lifetime restriction.
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let value: &str = &**attr.value();
            unsafe { mem::transmute(value) }
        })
    }
    #[allow(unsafe_code)]
    fn get_attrs(self, local_name: &Atom) -> Vec<&'a str> {
        let mut attributes = RootedVec::new();
        self.get_attributes(local_name, &mut attributes);
        attributes.iter().map(|attr| attr.root()).map(|attr| {
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let value: &str = &**attr.value();
            // This transmute is used to cheat the lifetime restriction.
            unsafe { mem::transmute(value) }
        }).collect()
    }
    fn get_link(self) -> Option<&'a str> {
        // FIXME: This is HTML only.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        match node.type_id() {
            // https://html.spec.whatwg.org/multipage/#selector-link
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
    fn get_focus_state(self) -> bool {
        // TODO: Also check whether the top-level browsing context has the system focus,
        // and whether this element is a browsing context container.
        // https://html.spec.whatwg.org/multipage/#selector-focus
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_focus_state()
    }
    fn get_id(self) -> Option<Atom> {
        self.get_attribute(&ns!(""), &atom!("id")).map(|attr| {
            let attr = attr.root();
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attr = attr.r();
            let value = attr.value();
            match *value {
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
        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("class")).root() {
            if let Some(tokens) = attr.r().value().tokens() {
                for token in tokens {
                    callback(token)
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
        let element = match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element: &'a JSRef<'a, HTMLInputElement> = HTMLInputElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &'a (Activatable + 'a))
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
                let element: &'a JSRef<'a, HTMLAnchorElement> = HTMLAnchorElementCast::to_borrowed_ref(self).unwrap();
                Some(element as &'a (Activatable + 'a))
            },
            _ => {
                None
            }
        };
        element.and_then(|elem| {
            if elem.is_instance_activatable() {
              Some(elem)
            } else {
              None
            }
        })
    }

    fn click_in_progress(self) -> bool {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.get_flag(CLICK_IN_PROGRESS)
    }

    fn set_click_in_progress(self, click: bool) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.set_flag(CLICK_IN_PROGRESS, click)
    }

    // https://html.spec.whatwg.org/multipage/#nearest-activatable-element
    fn nearest_activable_element(self) -> Option<Temporary<Element>> {
        match self.as_maybe_activatable() {
            Some(el) => Some(Temporary::from_rooted(el.as_element().root().r())),
            None => {
                let node: JSRef<Node> = NodeCast::from_ref(self);
                for node in node.ancestors() {
                    let node = node.root();
                    if let Some(node) = ElementCast::to_ref(node.r()) {
                        if node.as_maybe_activatable().is_some() {
                            return Some(Temporary::from_rooted(node))
                        }
                    }
                }
                None
            }
        }
    }

    /// Please call this method *only* for real click events
    ///
    /// https://html.spec.whatwg.org/multipage/#run-authentic-click-activation-steps
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
            Some(ref el) => match el.r().as_maybe_activatable() {
                Some(elem) => {
                    // Step 5-6
                    elem.pre_click_activation();
                    event.fire(target);
                    if !event.DefaultPrevented() {
                        // post click activation
                        elem.activation_behavior(event, target);
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
