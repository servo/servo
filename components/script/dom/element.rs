/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use app_units::Au;
use cssparser::Color;
use devtools_traits::AttrInfo;
use dom::activation::Activatable;
use dom::attr::AttrValue;
use dom::attr::{Attr, AttrHelpersForLayout};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::trace::JSTraceable;
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::bindings::xmlname::{namespace_from_domstring, validate_and_extract, xml_name_type};
use dom::characterdata::CharacterData;
use dom::create::create_element;
use dom::document::{Document, LayoutDocumentHelpers};
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::domtokenlist::DOMTokenList;
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementLayoutHelpers};
use dom::htmlcollection::HTMLCollection;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::{HTMLFontElement, HTMLFontElementLayoutHelpers};
use dom::htmlhrelement::{HTMLHRElement, HTMLHRLayoutHelpers};
use dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllegendelement::HTMLLegendElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementLayoutHelpers};
use dom::htmltableelement::{HTMLTableElement, HTMLTableElementLayoutHelpers};
use dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementLayoutHelpers};
use dom::htmltablesectionelement::{HTMLTableSectionElement, HTMLTableSectionElementLayoutHelpers};
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::htmltextareaelement::{HTMLTextAreaElement, RawLayoutHTMLTextAreaElementHelpers};
use dom::namednodemap::NamedNodeMap;
use dom::node::{CLICK_IN_PROGRESS, LayoutNodeHelpers, Node};
use dom::node::{NodeDamage, SEQUENTIALLY_FOCUSABLE};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::tree_builder::{LimitedQuirks, NoQuirks, Quirks};
use selectors::matching::{DeclarationBlock, matches};
use selectors::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use selectors::parser::{AttrSelector, NamespaceConstraint, parse_author_origin_selector_list_from_str};
use selectors::states::*;
use smallvec::VecLike;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::{Cell, Ref};
use std::default::Default;
use std::mem;
use std::sync::Arc;
use string_cache::{Atom, Namespace, QualName};
use style::properties::DeclaredValue;
use style::properties::longhands::{self, background_image, border_spacing, font_family, font_size};
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, parse_style_attribute};
use style::values::CSSFloat;
use style::values::specified::{self, CSSColor, CSSRGBA, LengthOrPercentage};
use url::UrlParser;
use util::mem::HeapSizeOf;
use util::str::{DOMString, LengthOrPercentageOrAuto};

// TODO: Update focus state when the top-level browsing context gains or loses system focus,
// and when the element enters or leaves a browsing context container.
// https://html.spec.whatwg.org/multipage/#selector-focus

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: Atom,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    id_attribute: DOMRefCell<Option<Atom>>,
    style_attribute: DOMRefCell<Option<PropertyDeclarationBlock>>,
    attr_list: MutNullableHeap<JS<NamedNodeMap>>,
    class_list: MutNullableHeap<JS<DOMTokenList>>,
    state: Cell<ElementState>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        self as *const Element == &*other
    }
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


    pub fn new_inherited(local_name: DOMString,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: &Document) -> Element {
        Element::new_inherited_with_state(ElementState::empty(), local_name,
                                          namespace, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, local_name: DOMString,
                                    namespace: Namespace, prefix: Option<DOMString>,
                                    document: &Document)
                                    -> Element {
        Element {
            node: Node::new_inherited(document),
            local_name: Atom::from_slice(&local_name),
            namespace: namespace,
            prefix: prefix,
            attrs: DOMRefCell::new(vec![]),
            id_attribute: DOMRefCell::new(None),
            style_attribute: DOMRefCell::new(None),
            attr_list: Default::default(),
            class_list: Default::default(),
            state: Cell::new(state),
        }
    }

    pub fn new(local_name: DOMString,
               namespace: Namespace,
               prefix: Option<DOMString>,
               document: &Document) -> Root<Element> {
        Node::reflect_node(
            box Element::new_inherited(local_name, namespace, prefix, document),
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
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn get_attr_for_layout<'a>(elem: &'a Element, namespace: &Namespace, name: &Atom)
                                      -> Option<LayoutJS<Attr>> {
    // cast to point to T in RefCell<T> directly
    let attrs = elem.attrs.borrow_for_layout();
    attrs.iter().find(|attr| {
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
        attrs.iter().filter_map(|attr| {
            let attr = attr.to_layout();
            if *name == attr.local_name_atom_forever() {
              Some(attr.value_ref_forever())
            } else {
              None
            }
        }).collect()
    }
}

pub trait LayoutElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool;
    #[allow(unsafe_code)]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]>;

    #[allow(unsafe_code)]
    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>;
    #[allow(unsafe_code)]
    unsafe fn get_colspan(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    fn id_attribute(&self) -> *const Option<Atom>;
    fn style_attribute(&self) -> *const Option<PropertyDeclarationBlock>;
    fn local_name(&self) -> &Atom;
    fn namespace(&self) -> &Namespace;
    fn get_checked_state_for_layout(&self) -> bool;
    fn get_indeterminate_state_for_layout(&self) -> bool;

    fn get_state_for_layout(&self) -> ElementState;
}

impl LayoutElementHelpers for LayoutJS<Element> {
    #[allow(unsafe_code)]
    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(""), &atom!("class")).map_or(false, |attr| {
            attr.value_tokens_forever().unwrap().iter().any(|atom| atom == name)
        })
    }

    #[allow(unsafe_code)]
    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(""), &atom!("class"))
            .map(|attr| attr.value_tokens_forever().unwrap())
    }

    #[allow(unsafe_code)]
    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        #[inline]
        fn from_declaration(rule: PropertyDeclaration) -> DeclarationBlock<Vec<PropertyDeclaration>> {
            DeclarationBlock::from_declarations(Arc::new(vec![rule]))
        }

        let bgcolor = if let Some(this) = self.downcast::<HTMLBodyElement>() {
            this.get_background_color()
        } else if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_background_color()
        } else if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_background_color()
        } else if let Some(this) = self.downcast::<HTMLTableRowElement>() {
            this.get_background_color()
        } else if let Some(this) = self.downcast::<HTMLTableSectionElement>() {
            this.get_background_color()
        } else {
            None
        };

        if let Some(color) = bgcolor {
            hints.push(from_declaration(
                PropertyDeclaration::BackgroundColor(DeclaredValue::Value(
                    CSSColor { parsed: Color::RGBA(color), authored: None }))));
        }

        let background = if let Some(this) = self.downcast::<HTMLBodyElement>() {
            this.get_background()
        } else {
            None
        };

        if let Some(url) = background {
            hints.push(from_declaration(
                PropertyDeclaration::BackgroundImage(DeclaredValue::Value(
                    background_image::SpecifiedValue(Some(specified::Image::Url(url)))))));
        }

        let color = if let Some(this) = self.downcast::<HTMLFontElement>() {
            this.get_color()
        } else if let Some(this) = self.downcast::<HTMLBodyElement>() {
            // https://html.spec.whatwg.org/multipage/#the-page:the-body-element-20
            this.get_color()
        } else if let Some(this) = self.downcast::<HTMLHRElement>() {
            // https://html.spec.whatwg.org/multipage/#the-hr-element-2:presentational-hints-5
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

        let font_family = if let Some(this) = self.downcast::<HTMLFontElement>() {
            this.get_face()
        } else {
            None
        };

        if let Some(font_family) = font_family {
            hints.push(from_declaration(
                PropertyDeclaration::FontFamily(
                    DeclaredValue::Value(
                        font_family::computed_value::T(vec![
                            font_family::computed_value::FontFamily::FamilyName(
                                font_family)])))));
        }

        let font_size = if let Some(this) = self.downcast::<HTMLFontElement>() {
            this.get_size()
        } else {
            None
        };

        if let Some(font_size) = font_size {
            hints.push(from_declaration(
                PropertyDeclaration::FontSize(
                    DeclaredValue::Value(
                        font_size::SpecifiedValue(
                            LengthOrPercentage::Length(font_size))))))
        }

        let cellspacing = if let Some(this) = self.downcast::<HTMLTableElement>() {
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


        let size = if let Some(this) = self.downcast::<HTMLInputElement>() {
            // FIXME(pcwalton): More use of atoms, please!
            // FIXME(Ms2ger): this is nonsense! Invalid values also end up as
            //                a text field
            match (*self.unsafe_get()).get_attr_val_for_layout(&ns!(""), &atom!("type")) {
                Some("text") | Some("password") => {
                    match this.get_size_for_layout() {
                        0 => None,
                        s => Some(s as i32),
                    }
                }
                _ => None,
            }
        } else {
            None
        };

        if let Some(size) = size {
            let value = specified::Length::ServoCharacterWidth(specified::CharacterWidth(size));
            hints.push(from_declaration(
                PropertyDeclaration::Width(DeclaredValue::Value(
                    specified::LengthOrPercentageOrAuto::Length(value)))));
        }


        let width = if let Some(this) = self.downcast::<HTMLIFrameElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLHRElement>() {
            // https://html.spec.whatwg.org/multipage/#the-hr-element-2:attr-hr-width
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


        let height = if let Some(this) = self.downcast::<HTMLIFrameElement>() {
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


        let cols = if let Some(this) = self.downcast::<HTMLTextAreaElement>() {
            match (*this.unsafe_get()).get_cols_for_layout() {
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


        let rows = if let Some(this) = self.downcast::<HTMLTextAreaElement>() {
            match (*this.unsafe_get()).get_rows_for_layout() {
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


        let border = if let Some(this) = self.downcast::<HTMLTableElement>() {
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

    #[allow(unsafe_code)]
    unsafe fn get_colspan(self) -> u32 {
        if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_colspan().unwrap_or(1)
        } else {
            // Don't panic since `display` can cause this to be called on arbitrary
            // elements.
            1
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool {
        if (*self.unsafe_get()).namespace != ns!(HTML) {
            return false;
        }
        self.upcast::<Node>().owner_doc_for_layout().is_html_document_for_layout()
    }

    #[allow(unsafe_code)]
    fn id_attribute(&self) -> *const Option<Atom> {
        unsafe {
            (*self.unsafe_get()).id_attribute.borrow_for_layout()
        }
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
        match self.downcast::<HTMLInputElement>() {
            Some(input) => unsafe {
                input.get_checked_state_for_layout()
            },
            None => false,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_indeterminate_state_for_layout(&self) -> bool {
        // TODO progress elements can also be matched with :indeterminate
        match self.downcast::<HTMLInputElement>() {
            Some(input) => unsafe {
                input.get_indeterminate_state_for_layout()
            },
            None => false,
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_state_for_layout(&self) -> ElementState {
        unsafe {
            (*self.unsafe_get()).state.get()
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
        self.namespace == ns!(HTML) && self.upcast::<Node>().is_in_html_doc()
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

    pub fn style_attribute(&self) -> &DOMRefCell<Option<PropertyDeclarationBlock>> {
        &self.style_attribute
    }

    pub fn summarize(&self) -> Vec<AttrInfo> {
        self.attrs.borrow().iter()
                           .map(|attr| attr.summarize())
                           .collect()
    }

    pub fn is_void(&self) -> bool {
        if self.namespace != ns!(HTML) {
            return false
        }
        match self.local_name {
            /* List of void elements from
            https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm */
            atom!(area) | atom!(base) | atom!(basefont) | atom!(bgsound) | atom!(br) | atom!(col) | atom!(embed) |
            atom!(frame) | atom!(hr) | atom!(img) | atom!(input) | atom!(keygen) | atom!(link) | atom!(menuitem) |
            atom!(meta) | atom!(param) | atom!(source) | atom!(track) | atom!(wbr) => true,
            _ => false
        }
    }

    pub fn remove_inline_style_property(&self, property: &str) {
        let mut inline_declarations = self.style_attribute.borrow_mut();
        if let &mut Some(ref mut declarations) = &mut *inline_declarations {
            let index = declarations.normal
                                    .iter()
                                    .position(|decl| decl.matches(property));
            if let Some(index) = index {
                Arc::make_mut(&mut declarations.normal).remove(index);
                return;
            }

            let index = declarations.important
                                    .iter()
                                    .position(|decl| decl.matches(property));
            if let Some(index) = index {
                Arc::make_mut(&mut declarations.important).remove(index);
                return;
            }
        }
    }

    pub fn update_inline_style(&self,
                               property_decl: PropertyDeclaration,
                               style_priority: StylePriority) {
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
            (vec![property_decl], vec![])
        } else {
            (vec![], vec![property_decl])
        };

        *inline_declarations = Some(PropertyDeclarationBlock {
            important: Arc::new(important),
            normal: Arc::new(normal),
        });
    }

    pub fn set_inline_style_property_priority(&self,
                                              properties: &[&str],
                                              style_priority: StylePriority) {
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
                let name = declaration.name();
                if properties.iter().any(|p| name == **p) {
                    to.push(declaration)
                } else {
                    new_from.push(declaration)
                }
            }
            mem::replace(from, new_from);
        }
    }

    pub fn get_inline_style_declaration(&self,
                                        property: &Atom)
                                        -> Option<Ref<PropertyDeclaration>> {
        Ref::filter_map(self.style_attribute.borrow(), |inline_declarations| {
            inline_declarations.as_ref().and_then(|declarations| {
                declarations.normal
                            .iter()
                            .chain(declarations.important.iter())
                            .find(|decl| decl.matches(&property))
            })
        })
    }

    pub fn get_important_inline_style_declaration(&self,
                                                  property: &Atom)
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
        let mut writer = vec![];
        match serialize(&mut writer,
                        &self.upcast::<Node>(),
                        SerializeOpts {
                            traversal_scope: traversal_scope,
                            ..Default::default()
                        }) {
            // FIXME(ajeffrey): Directly convert UTF8 to DOMString
            Ok(()) => Ok(DOMString::from(String::from_utf8(writer).unwrap())),
            Err(_) => panic!("Cannot serialize element"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#root-element
    pub fn get_root_element(&self) -> Root<Element> {
        self.upcast::<Node>()
            .inclusive_ancestors()
            .filter_map(Root::downcast)
            .last()
            .expect("We know inclusive_ancestors will return `self` which is an element")
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace-prefix
    pub fn lookup_prefix(&self, namespace: Namespace) -> Option<DOMString> {
        for node in self.upcast::<Node>().inclusive_ancestors() {
            match node.downcast::<Element>() {
                Some(element) => {
                    // Step 1.
                    if *element.namespace() == namespace {
                        if let Some(prefix) = element.GetPrefix() {
                            return Some(prefix);
                        }
                    }

                    // Step 2.
                    for attr in element.attrs.borrow().iter() {
                        if *attr.prefix() == Some(atom!("xmlns")) &&
                           **attr.value() == *namespace.0 {
                            return Some(attr.LocalName());
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
        let node = self.upcast::<Node>();
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
            _ => false,
        }
    }

    pub fn is_actually_disabled(&self) -> bool {
        let node = self.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) => {
                self.get_disabled_state()
            }
            // TODO:
            // an optgroup element that has a disabled attribute
            // a menuitem element that has a disabled attribute
            // a fieldset element that is a disabled fieldset
            _ => false,
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
        self.will_mutate_attr();
        let window = window_from_node(self);
        let in_empty_ns = namespace == ns!("");
        let attr = Attr::new(&window,
                             local_name,
                             value,
                             name,
                             namespace,
                             prefix,
                             Some(self));
        self.attrs.borrow_mut().push(JS::from_rooted(&attr));
        if in_empty_ns {
            vtable_for(self.upcast()).attribute_mutated(&attr, AttributeMutation::Set(None));
        }
    }

    pub fn get_attribute(&self, namespace: &Namespace, local_name: &Atom) -> Option<Root<Attr>> {
        self.attrs
            .borrow()
            .iter()
            .find(|attr| attr.local_name() == local_name && attr.namespace() == namespace)
            .map(|js| Root::from_ref(&**js))
    }

    // https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name
    pub fn get_attribute_by_name(&self, name: DOMString) -> Option<Root<Attr>> {
        let name = &self.parsed_name(name);
        self.attrs.borrow().iter().find(|a| a.name() == name).map(|js| Root::from_ref(&**js))
    }

    pub fn set_attribute_from_parser(&self,
                                     qname: QualName,
                                     value: DOMString,
                                     prefix: Option<Atom>) {
        // Don't set if the attribute already exists, so we can handle add_attrs_if_missing
        if self.attrs
               .borrow()
               .iter()
               .any(|a| *a.local_name() == qname.local && *a.namespace() == qname.ns) {
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

        self.set_first_matching_attribute(name.clone(),
                                          value,
                                          name.clone(),
                                          ns!(""),
                                          None,
                                          |attr| attr.local_name() == name);
    }

    // https://html.spec.whatwg.org/multipage/#attr-data-*
    pub fn set_custom_attribute(&self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        match xml_name_type(&name) {
            InvalidXMLName => return Err(Error::InvalidCharacter),
            _ => {}
        }

        // Steps 2-5.
        let name = Atom::from_slice(&name);
        let value = self.parse_attribute(&ns!(""), &name, value);
        self.set_first_matching_attribute(name.clone(),
                                          value,
                                          name.clone(),
                                          ns!(""),
                                          None,
                                          |attr| {
                                              *attr.name() == name && *attr.namespace() == ns!("")
                                          });
        Ok(())
    }

    fn set_first_matching_attribute<F>(&self,
                                       local_name: Atom,
                                       value: AttrValue,
                                       name: Atom,
                                       namespace: Namespace,
                                       prefix: Option<Atom>,
                                       find: F)
        where F: Fn(&Attr) -> bool
    {
        let attr = self.attrs
                       .borrow()
                       .iter()
                       .find(|attr| find(&attr))
                       .map(|js| Root::from_ref(&**js));
        if let Some(attr) = attr {
            attr.set_value(value, self);
        } else {
            self.push_new_attribute(local_name, value, name, namespace, prefix);
        };
    }

    pub fn parse_attribute(&self,
                           namespace: &Namespace,
                           local_name: &Atom,
                           value: DOMString)
                           -> AttrValue {
        if *namespace == ns!("") {
            vtable_for(self.upcast()).parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value)
        }
    }

    pub fn remove_attribute(&self, namespace: &Namespace, local_name: &Atom) -> Option<Root<Attr>> {
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
        let idx = self.attrs.borrow().iter().position(|attr| find(&attr));

        idx.map(|idx| {
            self.will_mutate_attr();
            let attr = Root::from_ref(&*(*self.attrs.borrow())[idx]);
            self.attrs.borrow_mut().remove(idx);
            attr.set_owner(None);
            if attr.namespace() == &ns!("") {
                vtable_for(self.upcast()).attribute_mutated(&attr, AttributeMutation::Removed);
            }
            attr
        })
    }

    pub fn has_class(&self, name: &Atom) -> bool {
        let quirks_mode = document_from_node(self).quirks_mode();
        let is_equal = |lhs: &Atom, rhs: &Atom| {
            match quirks_mode {
                NoQuirks | LimitedQuirks => lhs == rhs,
                Quirks => lhs.eq_ignore_ascii_case(&rhs),
            }
        };
        self.get_attribute(&ns!(""), &atom!("class"))
            .map(|attr| attr.value().as_tokens().iter().any(|atom| is_equal(name, atom)))
            .unwrap_or(false)
    }

    pub fn set_atomic_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value);
        self.set_attribute(local_name, value);
    }

    pub fn has_attribute(&self, local_name: &Atom) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        self.attrs
            .borrow()
            .iter()
            .any(|attr| attr.local_name() == local_name && attr.namespace() == &ns!(""))
    }

    pub fn set_bool_attribute(&self, local_name: &Atom, value: bool) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(local_name, DOMString::new());
        } else {
            self.remove_attribute(&ns!(""), local_name);
        }
    }

    pub fn get_url_attribute(&self, local_name: &Atom) -> DOMString {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        if !self.has_attribute(local_name) {
            return DOMString::new();
        }
        let url = self.get_string_attribute(local_name);
        let doc = document_from_node(self);
        let base = doc.url();
        // https://html.spec.whatwg.org/multipage/#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match UrlParser::new().base_url(&base).parse(&url) {
            Ok(parsed) => DOMString::from(parsed.serialize()),
            Err(_) => DOMString::from(""),
        }
    }
    pub fn set_url_attribute(&self, local_name: &Atom, value: DOMString) {
        self.set_string_attribute(local_name, value);
    }

    pub fn get_string_attribute(&self, local_name: &Atom) -> DOMString {
        match self.get_attribute(&ns!(""), local_name) {
            Some(x) => x.Value(),
            None => DOMString::new(),
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
        assert!(local_name.chars().all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch));
        let attribute = self.get_attribute(&ns!(""), local_name);
        match attribute {
            Some(ref attribute) => {
                match *attribute.value() {
                    AttrValue::UInt(_, value) => value,
                    _ => panic!("Expected an AttrValue::UInt: implement parse_plain_attribute"),
                }
            }
            None => default,
        }
    }
    pub fn set_uint_attribute(&self, local_name: &Atom, value: u32) {
        assert!(&**local_name == local_name.to_ascii_lowercase());
        // FIXME(ajeffrey): Directly convert u32 to DOMString
        self.set_attribute(local_name,
                           AttrValue::UInt(DOMString::from(value.to_string()), value));
    }

    pub fn will_mutate_attr(&self) {
        let node = self.upcast::<Node>();
        node.owner_doc().element_attr_will_change(self);
    }
}

impl ElementMethods for Element {
    // https://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        Node::namespace_to_string(self.namespace.clone())
    }

    // https://dom.spec.whatwg.org/#dom-element-localname
    fn LocalName(&self) -> DOMString {
        // FIXME(ajeffrey): Convert directly from Atom to DOMString
        DOMString::from(&*self.local_name)
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
        DOMString::from(if self.html_element_in_html_document() {
            qualified_name.to_ascii_uppercase()
        } else {
            qualified_name.into_owned()
        })
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
        self.attr_list.or_init(|| NamedNodeMap::new(&window_from_node(self), self))
    }

    // https://dom.spec.whatwg.org/#dom-element-getattribute
    fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        self.GetAttributeNode(name)
            .map(|s| s.Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(&self,
                      namespace: Option<DOMString>,
                      local_name: DOMString)
                      -> Option<DOMString> {
        self.GetAttributeNodeNS(namespace, local_name)
            .map(|attr| attr.Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributenode
    fn GetAttributeNode(&self, name: DOMString) -> Option<Root<Attr>> {
        self.get_attribute_by_name(name)
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributenodens
    fn GetAttributeNodeNS(&self,
                          namespace: Option<DOMString>,
                          local_name: DOMString)
                          -> Option<Root<Attr>> {
        let namespace = &namespace_from_domstring(namespace);
        self.get_attribute(namespace, &Atom::from_slice(&local_name))
    }

    // https://dom.spec.whatwg.org/#dom-element-setattribute
    fn SetAttribute(&self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        if xml_name_type(&name) == InvalidXMLName {
            return Err(Error::InvalidCharacter);
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
    fn RemoveAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) {
        let namespace = namespace_from_domstring(namespace);
        let local_name = Atom::from_slice(&local_name);
        self.remove_attribute(&namespace, &local_name);
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattribute
    fn HasAttribute(&self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-hasattributens
    fn HasAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    fn GetElementsByTagName(&self, localname: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_tag_name(window.r(), self.upcast(), localname)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self,
                              maybe_ns: Option<DOMString>,
                              localname: DOMString)
                              -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_tag_name_ns(window.r(), self.upcast(), localname, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_class_name(window.r(), self.upcast(), classes)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self) -> Root<DOMRectList> {
        let win = window_from_node(self);
        let raw_rects = self.upcast::<Node>().get_content_boxes();
        let rects = raw_rects.iter().map(|rect| {
            DOMRect::new(GlobalRef::Window(win.r()),
                         rect.origin.x.to_f64_px(),
                         rect.origin.y.to_f64_px(),
                         rect.size.width.to_f64_px(),
                         rect.size.height.to_f64_px())
        });
        DOMRectList::new(win.r(), rects)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self) -> Root<DOMRect> {
        let win = window_from_node(self);
        let rect = self.upcast::<Node>().get_bounding_content_box();
        DOMRect::new(GlobalRef::Window(win.r()),
                     rect.origin.x.to_f64_px(),
                     rect.origin.y.to_f64_px(),
                     rect.size.width.to_f64_px(),
                     rect.size.height.to_f64_px())
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
    fn ClientTop(&self) -> i32 {
        self.upcast::<Node>().get_client_rect().origin.y
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientleft
    fn ClientLeft(&self) -> i32 {
        self.upcast::<Node>().get_client_rect().origin.x
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientwidth
    fn ClientWidth(&self) -> i32 {
        self.upcast::<Node>().get_client_rect().size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientheight
    fn ClientHeight(&self) -> i32 {
        self.upcast::<Node>().get_client_rect().size.height
    }

    /// https://w3c.github.io/DOM-Parsing/#widl-Element-innerHTML
    fn GetInnerHTML(&self) -> Fallible<DOMString> {
        // XXX TODO: XML case
        self.serialize(ChildrenOnly)
    }

    /// https://w3c.github.io/DOM-Parsing/#widl-Element-innerHTML
    fn SetInnerHTML(&self, value: DOMString) -> Fallible<()> {
        let context_node = self.upcast::<Node>();
        // Step 1.
        let frag = try!(context_node.parse_fragment(value));
        // Step 2.
        // https://github.com/w3c/DOM-Parsing/issues/1
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            Root::upcast(template.Content())
        } else {
            Root::from_ref(context_node)
        };
        Node::replace_all(Some(frag.upcast()), &target);
        Ok(())
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn GetOuterHTML(&self) -> Fallible<DOMString> {
        self.serialize(IncludeNode)
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn SetOuterHTML(&self, value: DOMString) -> Fallible<()> {
        let context_document = document_from_node(self);
        let context_node = self.upcast::<Node>();
        // Step 1.
        let context_parent = match context_node.GetParentNode() {
            None => {
                // Step 2.
                return Ok(());
            },
            Some(parent) => parent,
        };

        let parent = match context_parent.type_id() {
            // Step 3.
            NodeTypeId::Document => return Err(Error::NoModificationAllowed),

            // Step 4.
            NodeTypeId::DocumentFragment => {
                let body_elem = Element::create(QualName::new(ns!(HTML), atom!(body)),
                                                None, context_document.r(),
                                                ElementCreator::ScriptCreated);
                Root::upcast(body_elem)
            },
            _ => context_node.GetParentNode().unwrap()
        };

        // Step 5.
        let frag = try!(parent.parse_fragment(value));
        // Step 6.
        try!(context_parent.ReplaceChild(frag.upcast(), context_node));
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().preceding_siblings().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().following_siblings().filter_map(Root::downcast).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::children(window.r(), self.upcast())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().rev_children().filter_map(Root::downcast::<Element>).next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().before(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().after(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_with(nodes)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self) {
        self.upcast::<Node>().remove_self();
    }

    // https://dom.spec.whatwg.org/#dom-element-matches
    fn Matches(&self, selectors: DOMString) -> Fallible<bool> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Error::Syntax),
            Ok(ref selectors) => {
                Ok(matches(selectors, &Root::from_ref(self), None))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-webkitmatchesselector
    fn WebkitMatchesSelector(&self, selectors: DOMString) -> Fallible<bool> {
        self.Matches(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        match parse_author_origin_selector_list_from_str(&selectors) {
            Err(()) => Err(Error::Syntax),
            Ok(ref selectors) => {
                let root = self.upcast::<Node>();
                for element in root.inclusive_ancestors() {
                    if let Some(element) = Root::downcast::<Element>(element) {
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

pub fn fragment_affecting_attributes() -> [Atom; 3] {
    [atom!("width"), atom!("height"), atom!("src")]
}

impl VirtualMethods for Element {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<Node>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        let node = self.upcast::<Node>();
        let doc = node.owner_doc();
        match attr.local_name() {
            &atom!(style) => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() =
                    mutation.new_value(attr).map(|value| {
                        parse_style_attribute(&value, &doc.base_url())
                    });
                if node.is_in_doc() {
                    doc.content_changed(node, NodeDamage::NodeStyleDamaged);
                }
            },
            &atom!(id) => {
                *self.id_attribute.borrow_mut() =
                    mutation.new_value(attr).and_then(|value| {
                        let value = value.as_atom();
                        if value != &atom!("") {
                            Some(value.clone())
                        } else {
                            None
                        }
                    });
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
            },
            _ if attr.namespace() == &ns!("") => {
                if fragment_affecting_attributes().iter().any(|a| a == attr.local_name()) ||
                   common_style_affecting_attributes().iter().any(|a| &a.atom == attr.local_name()) ||
                   rare_style_affecting_attributes().iter().any(|a| a == attr.local_name())
                {
                    doc.content_changed(node, NodeDamage::OtherNodeDamage);
                }
            },
            _ => {},
        };

        // Make sure we rev the version even if we didn't dirty the node. If we
        // don't do this, various attribute-dependent htmlcollections (like those
        // generated by getElementsByClassName) might become stale.
        node.rev_version();
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

        if !tree_in_doc {
            return;
        }

        if let Some(ref value) = *self.id_attribute.borrow() {
            let doc = document_from_node(self);
            doc.register_named_element(self, value.clone());
        }
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        if !tree_in_doc {
            return;
        }

        if let Some(ref value) = *self.id_attribute.borrow() {
            let doc = document_from_node(self);
            doc.unregister_named_element(self, value.clone());
        }
    }
}

macro_rules! state_getter {
    ($(
        $(#[$Flag_attr: meta])*
        state $css: expr => $variant: ident / $method: ident /
        $flag: ident = $value: expr,
    )+) => {
        $( fn $method(&self) -> bool { Element::get_state(self).contains($flag) } )+
    }
}

impl<'a> ::selectors::Element for Root<Element> {
    fn parent_element(&self) -> Option<Root<Element>> {
        self.upcast::<Node>().GetParentElement()
    }

    fn first_child_element(&self) -> Option<Root<Element>> {
        self.node.child_elements().next()
    }

    fn last_child_element(&self) -> Option<Root<Element>> {
        self.node.rev_children().filter_map(Root::downcast).next()
    }

    fn prev_sibling_element(&self) -> Option<Root<Element>> {
        self.node.preceding_siblings().filter_map(Root::downcast).next()
    }

    fn next_sibling_element(&self) -> Option<Root<Element>> {
        self.node.following_siblings().filter_map(Root::downcast).next()
    }

    fn is_root(&self) -> bool {
        match self.node.GetParentNode() {
            None => false,
            Some(node) => node.is::<Document>(),
        }
    }

    fn is_empty(&self) -> bool {
        self.node.children().all(|node| !node.is::<Element>() && match node.downcast::<Text>() {
            None => true,
            Some(text) => text.upcast::<CharacterData>().data().is_empty()
        })
    }

    fn is_link(&self) -> bool {
        // FIXME: This is HTML only.
        let node = self.upcast::<Node>();
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

    fn get_local_name(&self) -> &Atom {
        self.local_name()
    }
    fn get_namespace(&self) -> &Namespace {
        self.namespace()
    }

    state_pseudo_classes!(state_getter);

    fn get_id(&self) -> Option<Atom> {
        self.id_attribute.borrow().clone()
    }
    fn has_class(&self, name: &Atom) -> bool {
        Element::has_class(&**self, name)
    }
    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        if let Some(ref attr) = self.get_attribute(&ns!(""), &atom!("class")) {
            let tokens = attr.value();
            let tokens = tokens.as_tokens();
            for token in tokens {
                callback(token);
            }
        }
    }
    fn has_servo_nonzero_border(&self) -> bool {
        match self.downcast::<HTMLTableElement>() {
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
                        test(&attr.value())
                    })
            },
            NamespaceConstraint::Any => {
                self.attrs.borrow().iter().any(|attr| {
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
    pub fn as_maybe_activatable(&self) -> Option<&Activatable> {
        let element = match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element = self.downcast::<HTMLInputElement>().unwrap();
                Some(element as &Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) => {
                let element = self.downcast::<HTMLAnchorElement>().unwrap();
                Some(element as &Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLabelElement)) => {
                let element = self.downcast::<HTMLLabelElement>().unwrap();
                Some(element as &Activatable)
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
        self.upcast::<Node>().get_flag(CLICK_IN_PROGRESS)
    }

    pub fn set_click_in_progress(&self, click: bool) {
        self.upcast::<Node>().set_flag(CLICK_IN_PROGRESS, click)
    }

    // https://html.spec.whatwg.org/multipage/#nearest-activatable-element
    pub fn nearest_activable_element(&self) -> Option<Root<Element>> {
        match self.as_maybe_activatable() {
            Some(el) => Some(Root::from_ref(el.as_element())),
            None => {
                let node = self.upcast::<Node>();
                for node in node.ancestors() {
                    if let Some(node) = node.downcast::<Element>() {
                        if node.as_maybe_activatable().is_some() {
                            return Some(Root::from_ref(node));
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
    pub fn authentic_click_activation(&self, event: &Event) {
        // Not explicitly part of the spec, however this helps enforce the invariants
        // required to save state between pre-activation and post-activation
        // since we cannot nest authentic clicks (unlike synthetic click activation, where
        // the script can generate more click events from the handler)
        assert!(!self.click_in_progress());

        let target = self.upcast();
        // Step 2 (requires canvas support)
        // Step 3
        self.set_click_in_progress(true);
        // Step 4
        let e = self.nearest_activable_element();
        match e {
            Some(ref el) => match el.as_maybe_activatable() {
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
                None => {
                    event.fire(target);
                }
            },
            // Step 6
            None => {
                event.fire(target);
            }
        }
        // Step 7
        self.set_click_in_progress(false);
    }

    pub fn get_state(&self) -> ElementState {
        self.state.get()
    }

    pub fn set_state(&self, which: ElementState, value: bool) {
        let mut state = self.state.get();
        if state.contains(which) == value {
            return;
        }
        let node = self.upcast::<Node>();
        node.owner_doc().element_state_will_change(self);
        match value {
            true => state.insert(which),
            false => state.remove(which),
        };
        self.state.set(state);
    }

    pub fn get_active_state(&self) -> bool {
        self.state.get().contains(IN_ACTIVE_STATE)
    }

    pub fn set_active_state(&self, value: bool) {
        self.set_state(IN_ACTIVE_STATE, value)
    }

    pub fn get_focus_state(&self) -> bool {
        self.state.get().contains(IN_FOCUS_STATE)
    }

    pub fn set_focus_state(&self, value: bool) {
        self.set_state(IN_FOCUS_STATE, value)
    }

    pub fn get_hover_state(&self) -> bool {
        self.state.get().contains(IN_HOVER_STATE)
    }

    pub fn set_hover_state(&self, value: bool) {
        self.set_state(IN_HOVER_STATE, value)
    }

    pub fn get_enabled_state(&self) -> bool {
        self.state.get().contains(IN_ENABLED_STATE)
    }

    pub fn set_enabled_state(&self, value: bool) {
        self.set_state(IN_ENABLED_STATE, value)
    }

    pub fn get_disabled_state(&self) -> bool {
        self.state.get().contains(IN_DISABLED_STATE)
    }

    pub fn set_disabled_state(&self, value: bool) {
        self.set_state(IN_DISABLED_STATE, value)
    }
}

impl Element {
    pub fn check_ancestors_disabled_state_for_form_control(&self) {
        let node = self.upcast::<Node>();
        if self.get_disabled_state() {
            return;
        }
        for ancestor in node.ancestors() {
            let ancestor = ancestor;
            let ancestor = ancestor.r();
            if !ancestor.is::<HTMLFieldSetElement>() {
                continue;
            }
            if !ancestor.downcast::<Element>().unwrap().get_disabled_state() {
                continue;
            }
            if ancestor.is_parent_of(node) {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
                return;
            }
            match ancestor.children()
                          .find(|child| child.is::<HTMLLegendElement>()) {
                Some(ref legend) => {
                    // XXXabinader: should we save previous ancestor to avoid this iteration?
                    if node.ancestors().any(|ancestor| ancestor == *legend) {
                        continue;
                    }
                },
                None => (),
            }
            self.set_disabled_state(true);
            self.set_enabled_state(false);
            return;
        }
    }

    pub fn check_parent_disabled_state_for_option(&self) {
        if self.get_disabled_state() {
            return;
        }
        let node = self.upcast::<Node>();
        if let Some(ref parent) = node.GetParentNode() {
            if parent.is::<HTMLOptGroupElement>() &&
               parent.downcast::<Element>().unwrap().get_disabled_state() {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
            }
        }
    }

    pub fn check_disabled_attribute(&self) {
        let has_disabled_attrib = self.has_attribute(&atom!("disabled"));
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AttributeMutation<'a> {
    /// The attribute is set, keep track of old value.
    /// https://dom.spec.whatwg.org/#attribute-is-set
    Set(Option<&'a AttrValue>),

    /// The attribute is removed.
    /// https://dom.spec.whatwg.org/#attribute-is-removed
    Removed,
}

impl<'a> AttributeMutation<'a> {
    pub fn new_value<'b>(&self, attr: &'b Attr) -> Option<Ref<'b, AttrValue>> {
        match *self {
            AttributeMutation::Set(_) => Some(attr.value()),
            AttributeMutation::Removed => None,
        }
    }
}
