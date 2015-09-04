/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::attr::{Attr, AttrHelpersForLayout};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::CharacterDataCast;
use dom::bindings::codegen::InheritTypes::DocumentDerived;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementCast;
use dom::bindings::codegen::InheritTypes::TextCast;
use dom::bindings::codegen::InheritTypes::{ElementCast, ElementDerived, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLBodyElementDerived, HTMLFontElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLIFrameElementDerived, HTMLInputElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLTableElementCast};
use dom::bindings::codegen::InheritTypes::{HTMLTableElementDerived, HTMLTableCellElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableRowElementDerived, HTMLTextAreaElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableSectionElementDerived, NodeCast};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::Error::NoModificationAllowed;
use dom::bindings::error::Error::{InvalidCharacter, Syntax};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::utils::XMLName::InvalidXMLName;
use dom::bindings::utils::{namespace_from_domstring, xml_name_type, validate_and_extract};
use dom::create::create_element;
use dom::document::{Document, LayoutDocumentHelpers};
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::domtokenlist::DOMTokenList;
use dom::event::Event;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElementTypeId;
use dom::htmlfontelement::HTMLFontElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlinputelement::{HTMLInputElement, RawLayoutHTMLInputElementHelpers};
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::htmltableelement::HTMLTableElement;
use dom::htmltablerowelement::HTMLTableRowElement;
use dom::htmltablesectionelement::HTMLTableSectionElement;
use dom::htmltextareaelement::{HTMLTextAreaElement, RawLayoutHTMLTextAreaElementHelpers};
use dom::namednodemap::NamedNodeMap;
use dom::node::{CLICK_IN_PROGRESS, LayoutNodeHelpers, Node, NodeTypeId, SEQUENTIALLY_FOCUSABLE};
use dom::node::{document_from_node, NodeDamage};
use dom::node::{window_from_node};
use dom::nodelist::NodeList;
use dom::virtualmethods::{VirtualMethods, vtable_for};

use devtools_traits::AttrInfo;
use smallvec::VecLike;
use style::legacy::{UnsignedIntegerAttribute, from_declaration};
use style::properties::DeclaredValue;
use style::properties::longhands::{self, background_image, border_spacing};
use style::properties::{PropertyDeclarationBlock, PropertyDeclaration, parse_style_attribute};
use style::values::CSSFloat;
use style::values::specified::{self, CSSColor, CSSRGBA};
use util::geometry::Au;
use util::str::{DOMString, LengthOrPercentageOrAuto};

use cssparser::Color;
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{IncludeNode, ChildrenOnly};
use html5ever::tree_builder::{NoQuirks, LimitedQuirks, Quirks};
use selectors::matching::{matches, DeclarationBlock};
use selectors::parser::parse_author_origin_selector_list_from_str;
use selectors::parser::{AttrSelector, NamespaceConstraint};
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

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self as *const Element == &*other
    }
}

#[derive(JSTraceable, Copy, Clone, PartialEq, Debug, HeapSizeOf)]
pub enum ElementTypeId {
    HTMLElement(HTMLElementTypeId),
    Element,
}

#[derive(PartialEq, HeapSizeOf)]
pub enum ElementCreator {
    ParserCreated,
    ScriptCreated,
}

//
// Element methods
//
impl Element {
    pub fn create(name: QualName, prefix: Option<Atom>,
                  document: &Document, creator: ElementCreator)
                  -> Root<Element> {
        create_element(name, prefix, document, creator)
    }

    pub fn new_inherited(type_id: ElementTypeId, local_name: DOMString,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: &Document) -> Element {
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

    pub fn new(local_name: DOMString,
               namespace: Namespace,
               prefix: Option<DOMString>,
               document: &Document) -> Root<Element> {
        Node::reflect_node(
            box Element::new_inherited(ElementTypeId::Element, local_name, namespace, prefix, document),
            document,
            ElementBinding::Wrap)
    }
}

#[allow(unsafe_code)]
pub trait RawLayoutElementHelpers {
    unsafe fn get_attr_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                      -> Option<&'a AttrValue>;
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                      -> Option<&'a str>;
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str>;
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom) -> Option<Atom>;
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool;
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]>;

    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    unsafe fn get_unsigned_integer_attribute_for_layout(&self, attribute: UnsignedIntegerAttribute)
                                                        -> Option<u32>;
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn get_attr_for_layout<'a>(elem: &'a Element, namespace: &Namespace, name: &Atom)
                                      -> Option<LayoutJS<Attr>> {
    // cast to point to T in RefCell<T> directly
    let attrs = elem.attrs.borrow_for_layout();
    attrs.iter().find(|attr: & &JS<Attr>| {
        let attr = attr.to_layout();
        *name == attr.local_name_atom_forever() &&
        (*attr.unsafe_get()).namespace() == namespace
    }).map(|attr| attr.to_layout())
}

#[allow(unsafe_code)]
impl RawLayoutElementHelpers for Element {
    #[inline]
    unsafe fn get_attr_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                      -> Option<&'a AttrValue> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            attr.value_forever()
        })
    }

    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &Atom)
                                          -> Option<&'a str> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            attr.value_ref_forever()
        })
    }

    #[inline]
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &Atom) -> Vec<&'a str> {
        let attrs = self.attrs.borrow_for_layout();
        (*attrs).iter().filter_map(|attr: &JS<Attr>| {
            let attr = attr.to_layout();
            if *name == attr.local_name_atom_forever() {
              Some(attr.value_ref_forever())
            } else {
              None
            }
        }).collect()
    }

    #[inline]
    unsafe fn get_attr_atom_for_layout(&self, namespace: &Namespace, name: &Atom)
                                      -> Option<Atom> {
        get_attr_for_layout(self, namespace, name).and_then(|attr| {
            attr.value_atom_forever()
        })
    }

    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        get_attr_for_layout(self, &ns!(""), &atom!("class")).map_or(false, |attr| {
            attr.value_tokens_forever().unwrap().iter().any(|atom| atom == name)
        })
    }

    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        get_attr_for_layout(self, &ns!(""), &atom!("class")).map(|attr| {
            attr.value_tokens_forever().unwrap()
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
        };

        if let Some(color) = bgcolor {
            hints.push(from_declaration(
                PropertyDeclaration::BackgroundColor(DeclaredValue::Value(
                    CSSColor { parsed: Color::RGBA(color), authored: None }))));
        }

        let background = if self.is_htmlbodyelement() {
            let this: &HTMLBodyElement = mem::transmute(self);
            this.get_background()
        } else {
            None
        };

        if let Some(url) = background {
            hints.push(from_declaration(
                PropertyDeclaration::BackgroundImage(DeclaredValue::Value(
                    background_image::SpecifiedValue(Some(specified::Image::Url(url)))))));
        }

        let color = if self.is_htmlfontelement() {
            let this: &HTMLFontElement = mem::transmute(self);
            this.get_color()
        } else {
            None
        };

        if let Some(color) = color {
            hints.push(from_declaration(
                PropertyDeclaration::Color(DeclaredValue::Value(CSSRGBA {
                    parsed: color,
                    authored: None,
                }))));
        }

        let cellspacing = if self.is_htmltableelement() {
            let this: &HTMLTableElement = mem::transmute(self);
            this.get_cellspacing()
        } else {
            None
        };

        if let Some(cellspacing) = cellspacing {
            let width_value = specified::Length::Absolute(Au::from_px(cellspacing as i32));
            hints.push(from_declaration(
                PropertyDeclaration::BorderSpacing(DeclaredValue::Value(
                    border_spacing::SpecifiedValue {
                        horizontal: width_value,
                        vertical: width_value,
                    }))));
        }


        let size = if self.is_htmlinputelement() {
            // FIXME(pcwalton): More use of atoms, please!
            // FIXME(Ms2ger): this is nonsense! Invalid values also end up as
            //                a text field
            match self.get_attr_val_for_layout(&ns!(""), &atom!("type")) {
                Some("text") | Some("password") => {
                    let this: &HTMLInputElement = mem::transmute(self);
                    match this.get_size_for_layout() {
                        0 => None,
                        s => Some(s as i32),
                    }
                }
                _ => None
            }
        } else {
            None
        };

        if let Some(size) = size {
            let value = specified::Length::ServoCharacterWidth(
                specified::CharacterWidth(size));
            hints.push(from_declaration(
                PropertyDeclaration::Width(DeclaredValue::Value(
                    specified::LengthOrPercentageOrAuto::Length(value)))));
        }


        let width = if self.is_htmliframeelement() {
            let this: &HTMLIFrameElement = mem::transmute(self);
            this.get_width()
        } else if self.is_htmltableelement() {
            let this: &HTMLTableElement = mem::transmute(self);
            this.get_width()
        } else if self.is_htmltablecellelement() {
            let this: &HTMLTableCellElement = mem::transmute(self);
            this.get_width()
        } else {
            LengthOrPercentageOrAuto::Auto
        };

        match width {
            LengthOrPercentageOrAuto::Auto => {}
            LengthOrPercentageOrAuto::Percentage(percentage) => {
                let width_value =
                    specified::LengthOrPercentageOrAuto::Percentage(specified::Percentage(percentage));
                hints.push(from_declaration(
                    PropertyDeclaration::Width(DeclaredValue::Value(width_value))));
            }
            LengthOrPercentageOrAuto::Length(length) => {
                let width_value = specified::LengthOrPercentageOrAuto::Length(
                    specified::Length::Absolute(length));
                hints.push(from_declaration(
                    PropertyDeclaration::Width(DeclaredValue::Value(width_value))));
            }
        }


        let height = if self.is_htmliframeelement() {
            let this: &HTMLIFrameElement = mem::transmute(self);
            this.get_height()
        } else {
            LengthOrPercentageOrAuto::Auto
        };

        match height {
            LengthOrPercentageOrAuto::Auto => {}
            LengthOrPercentageOrAuto::Percentage(percentage) => {
                let height_value =
                    specified::LengthOrPercentageOrAuto::Percentage(specified::Percentage(percentage));
                hints.push(from_declaration(
                    PropertyDeclaration::Height(DeclaredValue::Value(height_value))));
            }
            LengthOrPercentageOrAuto::Length(length) => {
                let height_value = specified::LengthOrPercentageOrAuto::Length(
                    specified::Length::Absolute(length));
                hints.push(from_declaration(
                    PropertyDeclaration::Height(DeclaredValue::Value(height_value))));
            }
        }


        let cols = if self.is_htmltextareaelement() {
            let this: &HTMLTextAreaElement = mem::transmute(self);
            match this.get_cols_for_layout() {
                0 => None,
                c => Some(c as i32),
            }
        } else {
            None
        };

        if let Some(cols) = cols {
            // TODO(mttr) ServoCharacterWidth uses the size math for <input type="text">, but
            // the math for <textarea> is a little different since we need to take
            // scrollbar size into consideration (but we don't have a scrollbar yet!)
            //
            // https://html.spec.whatwg.org/multipage/#textarea-effective-width
            let value = specified::Length::ServoCharacterWidth(specified::CharacterWidth(cols));
            hints.push(from_declaration(
                PropertyDeclaration::Width(DeclaredValue::Value(
                    specified::LengthOrPercentageOrAuto::Length(value)))));
        }


        let rows = if self.is_htmltextareaelement() {
            let this: &HTMLTextAreaElement = mem::transmute(self);
            match this.get_rows_for_layout() {
                0 => None,
                r => Some(r as i32),
            }
        } else {
            None
        };

        if let Some(rows) = rows {
            // TODO(mttr) This should take scrollbar size into consideration.
            //
            // https://html.spec.whatwg.org/multipage/#textarea-effective-height
            let value = specified::Length::FontRelative(specified::FontRelativeLength::Em(rows as CSSFloat));
            hints.push(from_declaration(
                PropertyDeclaration::Height(DeclaredValue::Value(
                        specified::LengthOrPercentageOrAuto::Length(value)))));
        }


        let border = if self.is_htmltableelement() {
            let this: &HTMLTableElement = mem::transmute(self);
            this.get_border()
        } else {
            None
        };

        if let Some(border) = border {
            let width_value = specified::Length::Absolute(Au::from_px(border as i32));
            hints.push(from_declaration(
                PropertyDeclaration::BorderTopWidth(DeclaredValue::Value(
                    longhands::border_top_width::SpecifiedValue(width_value)))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderLeftWidth(DeclaredValue::Value(
                    longhands::border_left_width::SpecifiedValue(width_value)))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderBottomWidth(DeclaredValue::Value(
                    longhands::border_bottom_width::SpecifiedValue(width_value)))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderRightWidth(DeclaredValue::Value(
                    longhands::border_right_width::SpecifiedValue(width_value)))));
        }
    }

    unsafe fn get_unsigned_integer_attribute_for_layout(&self,
                                                        attribute: UnsignedIntegerAttribute)
                                                        -> Option<u32> {
        match attribute {
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
}

pub trait LayoutElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn has_attr_for_layout(&self, namespace: &Namespace, name: &Atom) -> bool;
    fn style_attribute(&self) -> *const Option<PropertyDeclarationBlock>;
    fn local_name(&self) -> &Atom;
    fn namespace(&self) -> &Namespace;
    fn get_checked_state_for_layout(&self) -> bool;
    fn get_indeterminate_state_for_layout(&self) -> bool;
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

    #[allow(unsafe_code)]
    fn style_attribute(&self) -> *const Option<PropertyDeclarationBlock> {
        unsafe {
            (*self.unsafe_get()).style_attribute.borrow_for_layout()
        }
    }

    #[allow(unsafe_code)]
    fn local_name(&self) -> &Atom {
        unsafe {
            &(*self.unsafe_get()).local_name
        }
    }

    #[allow(unsafe_code)]
    fn namespace(&self) -> &Namespace {
        unsafe {
            &(*self.unsafe_get()).namespace
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_checked_state_for_layout(&self) -> bool {
        // TODO option and menuitem can also have a checked state.
        match HTMLInputElementCast::to_layout_js(self) {
            Some(input) => unsafe {
                (*input.unsafe_get()).get_checked_state_for_layout()
            },
            None => false,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_indeterminate_state_for_layout(&self) -> bool {
        // TODO progress elements can also be matched with :indeterminate
        match HTMLInputElementCast::to_layout_js(self) {
            Some(input) => unsafe {
                (*input.unsafe_get()).get_indeterminate_state_for_layout()
            },
            None => false,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, HeapSizeOf)]
pub enum StylePriority {
    Important,
    Normal,
}


impl Element {
    pub fn html_element_in_html_document(&self) -> bool {
        let node = NodeCast::from_ref(self);
        self.namespace == ns!(HTML) && node.is_in_html_doc()
    }

    pub fn local_name(&self) -> &Atom {
        &self.local_name
    }

    pub fn parsed_name(&self, mut name: DOMString) -> Atom {
        if self.html_element_in_html_document() {
            name.make_ascii_lowercase();
        }
        Atom::from_slice(&name)
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn prefix(&self) -> &Option<DOMString> {
        &self.prefix
    }

    pub fn attrs(&self) -> Ref<Vec<JS<Attr>>> {
        self.attrs.borrow()
    }

    pub fn attrs_mut(&self) -> RefMut<Vec<JS<Attr>>> {
        self.attrs.borrow_mut()
    }

    pub fn style_attribute(&self) -> &DOMRefCell<Option<PropertyDeclarationBlock>> {
        &self.style_attribute
    }

    pub fn summarize(&self) -> Vec<AttrInfo> {
        let attrs = self.Attributes();
        let mut summarized = vec!();
        for i in 0..attrs.r().Length() {
            let attr = attrs.r().Item(i).unwrap();
            summarized.push(attr.r().summarize());
        }
        summarized
    }

    pub fn is_void(&self) -> bool {
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

    pub fn remove_inline_style_property(&self, property: &str) {
        let mut inline_declarations = self.style_attribute.borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let index = declarations.normal
                                    .iter()
                                    .position(|decl| decl.name() == property);
            if let Some(index) = index {
                Arc::make_mut(&mut declarations.normal).remove(index);
                return;
            }

            let index = declarations.important
                                    .iter()
                                    .position(|decl| decl.name() == property);
            if let Some(index) = index {
                Arc::make_mut(&mut declarations.important).remove(index);
                return;
            }
        }
    }

    pub fn update_inline_style(&self, property_decl: PropertyDeclaration, style_priority: StylePriority) {
        let mut inline_declarations = self.style_attribute().borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let existing_declarations = if style_priority == StylePriority::Important {
                &mut declarations.important
            } else {
                &mut declarations.normal
            };

            // Usually, the reference count will be 1 here. But transitions could make it greater
            // than that.
            let existing_declarations = Arc::make_mut(existing_declarations);
            for declaration in &mut *existing_declarations {
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

    pub fn set_inline_style_property_priority(&self, properties: &[&str], style_priority: StylePriority) {
        let mut inline_declarations = self.style_attribute().borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let (from, to) = if style_priority == StylePriority::Important {
                (&mut declarations.normal, &mut declarations.important)
            } else {
                (&mut declarations.important, &mut declarations.normal)
            };

            // Usually, the reference counts of `from` and `to` will be 1 here. But transitions
            // could make them greater than that.
            let from = Arc::make_mut(from);
            let to = Arc::make_mut(to);
            let mut new_from = Vec::new();
            for declaration in from.drain(..) {
                if properties.contains(&declaration.name()) {
                    to.push(declaration)
                } else {
                    new_from.push(declaration)
                }
            }
            mem::replace(from, new_from);
        }
    }

    pub fn get_inline_style_declaration(&self, property: &Atom) -> Option<Ref<PropertyDeclaration>> {
        Ref::filter_map(self.style_attribute.borrow(), |inline_declarations| {
            inline_declarations.as_ref().and_then(|declarations| {
                declarations.normal
                            .iter()
                            .chain(declarations.important.iter())
                            .find(|decl| decl.matches(&property))
            })
        })
    }

    pub fn get_important_inline_style_declaration(&self, property: &Atom)
                                                  -> Option<Ref<PropertyDeclaration>> {
        Ref::filter_map(self.style_attribute.borrow(), |inline_declarations| {
            inline_declarations.as_ref().and_then(|declarations| {
                declarations.important
                            .iter()
                            .find(|decl| decl.matches(&property))
            })
        })
    }

    pub fn serialize(&self, traversal_scope: TraversalScope) -> Fallible<DOMString> {
        let node = NodeCast::from_ref(self);
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
    pub fn get_root_element(&self) -> Root<Element> {
        let node = NodeCast::from_ref(self);
        node.inclusive_ancestors()
            .filter_map(ElementCast::to_root)
            .last()
            .expect("We know inclusive_ancestors will return `self` which is an element")
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace-prefix
    pub fn lookup_prefix(&self, namespace: Namespace) -> Option<DOMString> {
        for node in NodeCast::from_ref(self).inclusive_ancestors() {
            match ElementCast::to_ref(node.r()) {
                Some(element) => {
                    // Step 1.
                    if *element.namespace() == namespace {
                        if let Some(prefix) = element.GetPrefix() {
                            return Some(prefix);
                        }
                    }

                    // Step 2.
                    let attrs = element.Attributes();
                    for i in 0..attrs.r().Length() {
                        let attr = attrs.r().Item(i).unwrap();
                        if *attr.r().prefix() == Some(atom!("xmlns")) &&
                           **attr.r().value() == *namespace.0 {
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


impl Element {
    pub fn is_focusable_area(&self) -> bool {
        if self.is_actually_disabled() {
            return false;
        }
        // TODO: Check whether the element is being rendered (i.e. not hidden).
        let node = NodeCast::from_ref(self);
        if node.get_flag(SEQUENTIALLY_FOCUSABLE) {
            return true;
        }
        // https://html.spec.whatwg.org/multipage/#specially-focusable
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

    pub fn is_actually_disabled(&self) -> bool {
        let node = NodeCast::from_ref(self);
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


impl Element {
    pub fn push_new_attribute(&self,
                              local_name: Atom,
                              value: AttrValue,
                              name: Atom,
                              namespace: Namespace,
                              prefix: Option<Atom>) {
        let window = window_from_node(self);
        let in_empty_ns = namespace == ns!("");
        let attr = Attr::new(&window, local_name, value, name, namespace, prefix, Some(self));
        self.attrs.borrow_mut().push(JS::from_rooted(&attr));
        if in_empty_ns {
            vtable_for(NodeCast::from_ref(self)).attribute_mutated(
                &attr, AttributeMutation::Set(None));
        }
    }

    pub fn get_attribute(&self, namespace: &Namespace, local_name: &Atom) -> Option<Root<Attr>> {
        self.attrs.borrow().iter().map(JS::root).find(|attr| {
            attr.local_name() == local_name && attr.namespace() == namespace
        })
    }

    // https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name
    pub fn get_attribute_by_name(&self, name: DOMString) -> Option<Root<Attr>> {
        let name = &self.parsed_name(name);
        self.attrs.borrow().iter().map(JS::root)
             .find(|a| a.r().name() == name)
    }

    pub fn set_attribute_from_parser(&self,
                                     qname: QualName,
                                     value: DOMString,
                                     prefix: Option<Atom>) {
        // Don't set if the attribute already exists, so we can handle add_attrs_if_missing
        if self.attrs.borrow().iter().map(JS::root)
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
        self.push_new_attribute(qname.local, value, name, qname.ns, prefix);
    }

    pub fn set_attribute(&self, name: &Atom, value: AttrValue) {
        assert!(&**name == name.to_ascii_lowercase());
        assert!(!name.contains(":"));

        self.set_first_matching_attribute(
            name.clone(), value, name.clone(), ns!(""), None,
            |attr| attr.local_name() == name);
    }

    // https://html.spec.whatwg.org/multipage/#attr-data-*
    pub fn set_custom_attribute(&self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        match xml_name_type(&name) {
            InvalidXMLName => return Err(InvalidCharacter),
            _ => {}
        }

        // Steps 2-5.
        let name = Atom::from_slice(&name);
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.set_first_matching_attribute(
            name.clone(), value, name.clone(), ns!(""), None,
            |attr| *attr.name() == name && *attr.namespace() == ns!(""));
        Ok(())
    }

    fn set_first_matching_attribute<F>(&self,
                                       local_name: Atom,
                                       value: AttrValue,
                                       name: Atom,
                                       namespace: Namespace,
                                       prefix: Option<Atom>,
                                       find: F)
                                       where F: Fn(&Attr)
                                       -> bool {
        let attr = self.attrs.borrow().iter().map(JS::root).find(|attr| find(&attr));
        if let Some(attr) = attr {
            attr.set_value(value, self);
        } else {
            self.push_new_attribute(local_name, value, name, namespace, prefix);
        };
    }

    pub fn parse_attribute(&self, namespace: &Namespace, local_name: &Atom,
                       value: DOMString) -> AttrValue {
        if *namespace == ns!("") {
            vtable_for(&NodeCast::from_ref(self))
                .parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value)
        }
    }

    pub fn remove_attribute(&self, namespace: &Namespace, local_name: &Atom)
                        -> Option<Root<Attr>> {
        self.remove_first_matching_attribute(|attr| {
            attr.namespace() == namespace && attr.local_name() == local_name
        })
    }

    pub fn remove_attribute_by_name(&self, name: &Atom) -> Option<Root<Attr>> {
        self.remove_first_matching_attribute(|attr| attr.name() == name)
    }

    fn remove_first_matching_attribute<F>(&self, find: F) -> Option<Root<Attr>>
        where F: Fn(&Attr) -> bool
    {
        let idx = self.attrs.borrow().iter().map(JS::root).position(|attr| find(&attr));

        idx.map(|idx| {
            let attr = (*self.attrs.borrow())[idx].root();
            self.attrs.borrow_mut().remove(idx);
            attr.set_owner(None);
            let node = NodeCast::from_ref(self);
            if attr.namespace() == &ns!("") {
                vtable_for(node).attribute_mutated(&attr, AttributeMutation::Removed);
            }
            attr
        })
    }

    pub fn has_class(&self, name: &Atom) -> bool {
        let quirks_mode = {
            let node = NodeCast::from_ref(self);
            let owner_doc = node.owner_doc();
            owner_doc.r().quirks_mode()
        };
        let is_equal = |lhs: &Atom, rhs: &Atom| match quirks_mode {
            NoQuirks | LimitedQuirks => lhs == rhs,
            Quirks => lhs.eq_ignore_ascii_case(&rhs)
        };
        self.get_attribute(&ns!(""), &atom!("class")).map(|attr| {
            attr.r().value().as_tokens().iter().any(|atom| is_equal(name, atom))
        }).unwrap_or(false)
    }

    pub fn set_atomic_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value);
        self.set_attribute(local_name, value);
    }

    pub fn has_attribute(&self, local_name: &Atom) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        self.attrs.borrow().iter().map(JS::root).any(|attr| {
            attr.r().local_name() == local_name && attr.r().namespace() == &ns!("")
        })
    }

    pub fn set_bool_attribute(&self, local_name: &Atom, value: bool) {
        if self.has_attribute(local_name) == value { return; }
        if value {
            self.set_string_attribute(local_name, String::new());
        } else {
            self.remove_attribute(&ns!(""), local_name);
        }
    }

    pub fn get_url_attribute(&self, local_name: &Atom) -> DOMString {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        if !self.has_attribute(local_name) {
            return "".to_owned();
        }
        let url = self.get_string_attribute(local_name);
        let doc = document_from_node(self);
        let base = doc.r().url();
        // https://html.spec.whatwg.org/multipage/#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match UrlParser::new().base_url(&base).parse(&url) {
            Ok(parsed) => parsed.serialize(),
            Err(_) => "".to_owned()
        }
    }
    pub fn set_url_attribute(&self, local_name: &Atom, value: DOMString) {
        self.set_string_attribute(local_name, value);
    }

    pub fn get_string_attribute(&self, local_name: &Atom) -> DOMString {
        match self.get_attribute(&ns!(""), local_name) {
            Some(x) => x.r().Value(),
            None => "".to_owned()
        }
    }
    pub fn set_string_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value));
    }

    pub fn get_tokenlist_attribute(&self, local_name: &Atom) -> Vec<Atom> {
        self.get_attribute(&ns!(""), local_name).map(|attr| {
            attr.r()
                .value()
                .as_tokens()
                .to_vec()
        }).unwrap_or(vec!())
    }

    pub fn set_tokenlist_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_serialized_tokenlist(value));
    }

    pub fn set_atomic_tokenlist_attribute(&self, local_name: &Atom, tokens: Vec<Atom>) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens));
    }

    pub fn get_uint_attribute(&self, local_name: &Atom, default: u32) -> u32 {
        assert!(local_name.chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii_lowercase() == ch
        }));
        let attribute = self.get_attribute(&ns!(""), local_name);
        match attribute {
            Some(ref attribute) => {
                match *attribute.r().value() {
                    AttrValue::UInt(_, value) => value,
                    _ => panic!("Expected an AttrValue::UInt: \
                                 implement parse_plain_attribute"),
                }
            }
            None => default,
        }
    }
    pub fn set_uint_attribute(&self, local_name: &Atom, value: u32) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::UInt(value.to_string(), value));
    }
}

impl ElementMethods for Element {
    // https://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        Node::namespace_to_string(self.namespace.clone())
    }

    // https://dom.spec.whatwg.org/#dom-element-localname
    fn LocalName(&self) -> DOMString {
        (*self.local_name).to_owned()
    }

    // https://dom.spec.whatwg.org/#dom-element-prefix
    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.clone()
    }

    // https://dom.spec.whatwg.org/#dom-element-tagname
    fn TagName(&self) -> DOMString {
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
    fn Id(&self) -> DOMString {
        self.get_string_attribute(&atom!("id"))
    }

    // https://dom.spec.whatwg.org/#dom-element-id
    fn SetId(&self, id: DOMString) {
        self.set_atomic_attribute(&atom!("id"), id);
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(&self) -> DOMString {
        self.get_string_attribute(&atom!("class"))
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(&self, class: DOMString) {
        self.set_tokenlist_attribute(&atom!("class"), class);
    }

    // https://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(&self) -> Root<DOMTokenList> {
        self.class_list.or_init(|| DOMTokenList::new(self, &atom!("class")))
    }

    // https://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(&self) -> Root<NamedNodeMap> {
        self.attr_list.or_init(|| {
            let doc = {
                let node = NodeCast::from_ref(self);
                node.owner_doc()
            };
            let window = doc.r().window();
            NamedNodeMap::new(window.r(), self)
        })
    }

    // https://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        self.get_attribute_by_name(name)
                     .map(|s| s.r().Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(&self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> Option<DOMString> {
        let namespace = &namespace_from_domstring(namespace);
        self.get_attribute(namespace, &Atom::from_slice(&local_name))
                     .map(|attr| attr.r().Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(&self,
                    name: DOMString,
                    value: DOMString) -> ErrorResult {
        // Step 1.
        if xml_name_type(&name) == InvalidXMLName {
            return Err(InvalidCharacter);
        }

        // Step 2.
        let name = self.parsed_name(name);

        // Step 3-5.
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.set_first_matching_attribute(
            name.clone(), value, name.clone(), ns!(""), None,
            |attr| *attr.name() == name);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(&self,
                      namespace: Option<DOMString>,
                      qualified_name: DOMString,
                      value: DOMString) -> ErrorResult {
        let (namespace, prefix, local_name) =
            try!(validate_and_extract(namespace, &qualified_name));
        let qualified_name = Atom::from_slice(&qualified_name);
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.set_first_matching_attribute(
            local_name.clone(), value, qualified_name, namespace.clone(), prefix,
            |attr| *attr.local_name() == local_name && *attr.namespace() == namespace);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(&self, name: DOMString) {
        let name = self.parsed_name(name);
        self.remove_attribute_by_name(&name);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(&self,
                         namespace: Option<DOMString>,
                         local_name: DOMString) {
        let namespace = namespace_from_domstring(namespace);
        let local_name = Atom::from_slice(&local_name);
        self.remove_attribute(&namespace, &local_name);
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(&self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(&self,
                      namespace: Option<DOMString>,
                      local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    fn GetElementsByTagName(&self, localname: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_tag_name(window.r(), NodeCast::from_ref(self), localname)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self, maybe_ns: Option<DOMString>,
                              localname: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_tag_name_ns(window.r(), NodeCast::from_ref(self), localname, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_class_name(window.r(), NodeCast::from_ref(self), classes)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self) -> Root<DOMRectList> {
        let win = window_from_node(self);
        let node = NodeCast::from_ref(self);
        let raw_rects = node.get_content_boxes();
        let rects = raw_rects.iter().map(|rect| {
            DOMRect::new(win.r(),
                         rect.origin.y, rect.origin.y + rect.size.height,
                         rect.origin.x, rect.origin.x + rect.size.width)
        });
        DOMRectList::new(win.r(), rects)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self) -> Root<DOMRect> {
        let win = window_from_node(self);
        let node = NodeCast::from_ref(self);
        let rect = node.get_bounding_content_box();
        DOMRect::new(
            win.r(),
            rect.origin.y,
            rect.origin.y + rect.size.height,
            rect.origin.x,
            rect.origin.x + rect.size.width)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
    fn ClientTop(&self) -> i32 {
        let node = NodeCast::from_ref(self);
        node.get_client_rect().origin.y
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientleft
    fn ClientLeft(&self) -> i32 {
        let node = NodeCast::from_ref(self);
        node.get_client_rect().origin.x
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientwidth
    fn ClientWidth(&self) -> i32 {
        let node = NodeCast::from_ref(self);
        node.get_client_rect().size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientheight
    fn ClientHeight(&self) -> i32 {
        let node = NodeCast::from_ref(self);
        node.get_client_rect().size.height
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-innerHTML
    fn GetInnerHTML(&self) -> Fallible<DOMString> {
        //XXX TODO: XML case
        self.serialize(ChildrenOnly)
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-innerHTML
    fn SetInnerHTML(&self, value: DOMString) -> Fallible<()> {
        let context_node = NodeCast::from_ref(self);
        // Step 1.
        let frag = try!(context_node.parse_fragment(value));
        // Step 2.
        Node::replace_all(Some(NodeCast::from_ref(frag.r())), context_node);
        Ok(())
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn GetOuterHTML(&self) -> Fallible<DOMString> {
        self.serialize(IncludeNode)
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn SetOuterHTML(&self, value: DOMString) -> Fallible<()> {
        let context_document = document_from_node(self);
        let context_node = NodeCast::from_ref(self);
        // Step 1.
        let context_parent = match context_node.GetParentNode() {
            None => {
                // Step 2.
                return Ok(());
            },
            Some(parent) => parent,
        };

        let parent = match context_parent.r().type_id() {
            // Step 3.
            NodeTypeId::Document => return Err(NoModificationAllowed),

            // Step 4.
            NodeTypeId::DocumentFragment => {
                let body_elem = Element::create(QualName::new(ns!(HTML), atom!(body)),
                                                None, context_document.r(),
                                                ElementCreator::ScriptCreated);
                NodeCast::from_root(body_elem)
            },
            _ => context_node.GetParentNode().unwrap()
        };

        // Step 5.
        let frag = try!(parent.r().parse_fragment(value));
        // Step 6.
        try!(context_parent.r().ReplaceChild(NodeCast::from_ref(frag.r()),
                                             context_node));
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).preceding_siblings()
                                .filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).following_siblings()
                                .filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::children(window.r(), NodeCast::from_ref(self))
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(self).rev_children().filter_map(ElementCast::to_root).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        NodeCast::from_ref(self).child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = NodeCast::from_ref(self);
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = NodeCast::from_ref(self);
        root.query_selector_all(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        NodeCast::from_ref(self).replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        let node = NodeCast::from_ref(self);
        node.remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-element-matches
    fn Matches(&self, selectors: DOMString) -> Fallible<bool> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                Ok(matches(selectors, &Root::from_ref(self), None))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Syntax),
            Ok(ref selectors) => {
                let root = NodeCast::from_ref(self);
                for element in root.inclusive_ancestors() {
                    if let Some(element) = ElementCast::to_root(element) {
                        if matches(selectors, &element, None) {
                            return Ok(Some(element));
                        }
                    }
                }
                Ok(None)
            }
        }
    }
}

impl VirtualMethods for Element {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let node: &Node = NodeCast::from_ref(self);
        Some(node as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        let node = NodeCast::from_ref(self);
        let doc = node.owner_doc();
        let damage = match attr.local_name() {
            &atom!(style) => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() =
                    mutation.new_value(attr).map(|value| {
                        parse_style_attribute(&value, &doc.base_url())
                    });
                NodeDamage::NodeStyleDamaged
            },
            &atom!(class) => {
                // Modifying a class can change style.
                NodeDamage::NodeStyleDamaged
            },
            &atom!(id) => {
                if node.is_in_doc() {
                    let value = attr.value().as_atom().clone();
                    match mutation {
                        AttributeMutation::Set(old_value) => {
                            if let Some(old_value) = old_value {
                                let old_value = old_value.as_atom().clone();
                                doc.unregister_named_element(self, old_value);
                            }
                            if value != atom!("") {
                                doc.register_named_element(self, value);
                            }
                        },
                        AttributeMutation::Removed => {
                            if value != atom!("") {
                                doc.unregister_named_element(self, value);
                            }
                        }
                    }
                }
                NodeDamage::NodeStyleDamaged
            },
            _ => {
                // Modifying any other attribute might change arbitrary things.
                NodeDamage::OtherNodeDamage
            },
        };
        if node.is_in_doc() {
            doc.content_changed(node, damage);
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

        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("id")) {
            let value = attr.value();
            if !value.is_empty() {
                let doc = document_from_node(self);
                let value = Atom::from_slice(&value);
                doc.register_named_element(self, value.to_owned());
            }
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        if !tree_in_doc { return; }

        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("id")) {
            let value = attr.value();
            if !value.is_empty() {
                let doc = document_from_node(self);
                let value = Atom::from_slice(&value);
                doc.unregister_named_element(self, value.to_owned());
            }
        }
    }
}

impl<'a> ::selectors::Element for Root<Element> {
    fn parent_element(&self) -> Option<Root<Element>> {
        NodeCast::from_ref(&**self).GetParentElement()
    }

    fn first_child_element(&self) -> Option<Root<Element>> {
        self.node.child_elements().next()
    }

    fn last_child_element(&self) -> Option<Root<Element>> {
        self.node.rev_children().filter_map(ElementCast::to_root).next()
    }

    fn prev_sibling_element(&self) -> Option<Root<Element>> {
        self.node.preceding_siblings().filter_map(ElementCast::to_root).next()
    }

    fn next_sibling_element(&self) -> Option<Root<Element>> {
        self.node.following_siblings().filter_map(ElementCast::to_root).next()
    }

    fn is_root(&self) -> bool {
        match self.node.GetParentNode() {
            None => false,
            Some(node) => node.is_document(),
        }
    }

    fn is_empty(&self) -> bool {
        self.node.children().all(|node| !node.is_element() && match TextCast::to_ref(&*node) {
            None => true,
            Some(text) => CharacterDataCast::from_ref(text).data().is_empty()
        })
    }

    fn is_link(&self) -> bool {
        // FIXME: This is HTML only.
        let node = NodeCast::from_ref(&**self);
        match node.type_id() {
            // https://html.spec.whatwg.org/multipage/#selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
                self.has_attribute(&atom!("href"))
            },
            _ => false,
         }
    }

    #[inline]
    fn is_unvisited_link(&self) -> bool {
        self.is_link()
    }

    #[inline]
    fn is_visited_link(&self) -> bool {
        false
    }

    fn get_local_name<'b>(&'b self) -> &'b Atom {
        self.local_name()
    }
    fn get_namespace<'b>(&'b self) -> &'b Namespace {
        self.namespace()
    }
    fn get_hover_state(&self) -> bool {
        let node = NodeCast::from_ref(&**self);
        node.get_hover_state()
    }
    fn get_active_state(&self) -> bool {
        let node = NodeCast::from_ref(&**self);
        node.get_active_state()
    }
    fn get_focus_state(&self) -> bool {
        // TODO: Also check whether the top-level browsing context has the system focus,
        // and whether this element is a browsing context container.
        // https://html.spec.whatwg.org/multipage/#selector-focus
        let node = NodeCast::from_ref(&**self);
        node.get_focus_state()
    }
    fn get_id(&self) -> Option<Atom> {
        self.get_attribute(&ns!(""), &atom!("id")).map(|attr| {
            match *attr.r().value() {
                AttrValue::Atom(ref val) => val.clone(),
                _ => panic!("`id` attribute should be AttrValue::Atom"),
            }
        })
    }
    fn get_disabled_state(&self) -> bool {
        let node = NodeCast::from_ref(&**self);
        node.get_disabled_state()
    }
    fn get_enabled_state(&self) -> bool {
        let node = NodeCast::from_ref(&**self);
        node.get_enabled_state()
    }
    fn get_checked_state(&self) -> bool {
        let input_element: Option<&HTMLInputElement> = HTMLInputElementCast::to_ref(&**self);
        match input_element {
            Some(input) => input.Checked(),
            None => false,
        }
    }
    fn get_indeterminate_state(&self) -> bool {
        let input_element: Option<&HTMLInputElement> = HTMLInputElementCast::to_ref(&**self);
        match input_element {
            Some(input) => input.get_indeterminate_state(),
            None => false,
        }
    }
    fn has_class(&self, name: &Atom) -> bool {
        Element::has_class(&**self, name)
    }
    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("class")) {
            let tokens = attr.r().value();
            let tokens = tokens.as_tokens();
            for token in tokens {
                callback(token);
            }
        }
    }
    fn has_servo_nonzero_border(&self) -> bool {
        let table_element: Option<&HTMLTableElement> = HTMLTableElementCast::to_ref(&**self);
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

    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool
        where F: Fn(&str) -> bool
    {
        let local_name = {
            if self.is_html_element_in_html_document() {
                &attr.lower_name
            } else {
                &attr.name
            }
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attribute(ns, local_name)
                    .map_or(false, |attr| {
                        test(&attr.r().value())
                    })
            },
            NamespaceConstraint::Any => {
                self.attrs.borrow().iter().map(JS::root).any(|attr| {
                     attr.local_name() == local_name && test(&attr.value())
                })
            }
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.html_element_in_html_document()
    }
}


impl Element {
    pub fn as_maybe_activatable<'a>(&'a self) -> Option<&'a (Activatable + 'a)> {
        let node = NodeCast::from_ref(self);
        let element = match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element = HTMLInputElementCast::to_ref(self).unwrap();
                Some(element as &'a (Activatable + 'a))
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
                let element = HTMLAnchorElementCast::to_ref(self).unwrap();
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

    pub fn click_in_progress(&self) -> bool {
        let node = NodeCast::from_ref(self);
        node.get_flag(CLICK_IN_PROGRESS)
    }

    pub fn set_click_in_progress(&self, click: bool) {
        let node = NodeCast::from_ref(self);
        node.set_flag(CLICK_IN_PROGRESS, click)
    }

    // https://html.spec.whatwg.org/multipage/#nearest-activatable-element
    pub fn nearest_activable_element(&self) -> Option<Root<Element>> {
        match self.as_maybe_activatable() {
            Some(el) => Some(Root::from_ref(el.as_element())),
            None => {
                let node = NodeCast::from_ref(self);
                for node in node.ancestors() {
                    if let Some(node) = ElementCast::to_ref(node.r()) {
                        if node.as_maybe_activatable().is_some() {
                            return Some(Root::from_ref(node))
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
    pub fn authentic_click_activation<'b>(&self, event: &'b Event) {
        // Not explicitly part of the spec, however this helps enforce the invariants
        // required to save state between pre-activation and post-activation
        // since we cannot nest authentic clicks (unlike synthetic click activation, where
        // the script can generate more click events from the handler)
        assert!(!self.click_in_progress());

        let target = EventTargetCast::from_ref(self);
        // Step 2 (requires canvas support)
        // Step 3
        self.set_click_in_progress(true);
        // Step 4
        let e = self.nearest_activable_element();
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
                None => { event.fire(target); }
            },
            // Step 6
            None => { event.fire(target); }
        }
        // Step 7
        self.set_click_in_progress(false);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AttributeMutation<'a> {
    /// The attribute is set, keep track of old value.
    /// https://dom.spec.whatwg.org/#attribute-is-set
    Set(Option<&'a AttrValue>),

    /// The attribute is removed.
    /// https://dom.spec.whatwg.org/#attribute-is-removed
    Removed
}

impl<'a> AttributeMutation<'a> {
    pub fn new_value<'b>(&self, attr: &'b Attr) -> Option<Ref<'b, AttrValue>> {
        match *self {
            AttributeMutation::Set(_) => Some(attr.value()),
            AttributeMutation::Removed => None,
        }
    }
}
