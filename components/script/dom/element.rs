/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

use app_units::Au;
use cssparser::Color;
use devtools_traits::AttrInfo;
use dom::activation::Activatable;
use dom::attr::{Attr, AttrHelpersForLayout};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::WindowBinding::{ScrollBehavior, ScrollToOptions};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::bindings::xmlname::{namespace_from_domstring, validate_and_extract, xml_name_type};
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::characterdata::CharacterData;
use dom::create::create_element;
use dom::document::{Document, LayoutDocumentHelpers};
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::domtokenlist::DOMTokenList;
use dom::event::Event;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementLayoutHelpers};
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::{HTMLFontElement, HTMLFontElementLayoutHelpers};
use dom::htmlhrelement::{HTMLHRElement, HTMLHRLayoutHelpers};
use dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllegendelement::HTMLLegendElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementLayoutHelpers};
use dom::htmltableelement::{HTMLTableElement, HTMLTableElementLayoutHelpers};
use dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementLayoutHelpers};
use dom::htmltablesectionelement::{HTMLTableSectionElement, HTMLTableSectionElementLayoutHelpers};
use dom::htmltemplateelement::HTMLTemplateElement;
use dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use dom::namednodemap::NamedNodeMap;
use dom::node::{CLICK_IN_PROGRESS, ChildrenMutation, LayoutNodeHelpers, Node};
use dom::node::{NodeDamage, SEQUENTIALLY_FOCUSABLE, UnbindContext};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::text::Text;
use dom::validation::Validatable;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::tree_builder::{LimitedQuirks, NoQuirks, Quirks};
use parking_lot::RwLock;
use selectors::matching::{ElementFlags, MatchingReason, matches};
use selectors::matching::{HAS_EDGE_CHILD_SELECTOR, HAS_SLOW_SELECTOR, HAS_SLOW_SELECTOR_LATER_SIBLINGS};
use selectors::parser::{AttrSelector, NamespaceConstraint, parse_author_origin_selector_list_from_str};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::{Cell, Ref};
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use string_cache::{Atom, Namespace, QualName};
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::element_state::*;
use style::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use style::parser::ParserContextExtraData;
use style::properties::{DeclaredValue, Importance};
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock, parse_style_attribute};
use style::properties::longhands::{background_image, border_spacing, font_family, font_size, overflow_x};
use style::selector_impl::{NonTSPseudoClass, ServoSelectorImpl};
use style::selector_matching::ApplicableDeclarationBlock;
use style::sink::Push;
use style::values::CSSFloat;
use style::values::specified::{self, CSSColor, CSSRGBA, LengthOrPercentage};

// TODO: Update focus state when the top-level browsing context gains or loses system focus,
// and when the element enters or leaves a browsing context container.
// https://html.spec.whatwg.org/multipage/#selector-focus

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: Atom,
    tag_name: TagName,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    id_attribute: DOMRefCell<Option<Atom>>,
    #[ignore_heap_size_of = "Arc"]
    style_attribute: DOMRefCell<Option<Arc<RwLock<PropertyDeclarationBlock>>>>,
    attr_list: MutNullableHeap<JS<NamedNodeMap>>,
    class_list: MutNullableHeap<JS<DOMTokenList>>,
    state: Cell<ElementState>,
    atomic_flags: AtomicElementFlags,
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "<{}", self.local_name));
        if let Some(ref id) = *self.id_attribute.borrow() {
            try!(write!(f, " id={}", id));
        }
        write!(f, ">")
    }
}

#[derive(PartialEq, HeapSizeOf)]
pub enum ElementCreator {
    ParserCreated,
    ScriptCreated,
}

pub enum AdjacentPosition {
    BeforeBegin,
    AfterEnd,
    AfterBegin,
    BeforeEnd,
}

impl<'a> TryFrom<&'a str> for AdjacentPosition {
    type Err = Error;

    fn try_from(position: &'a str) -> Result<AdjacentPosition, Self::Err> {
        match_ignore_ascii_case! { &*position,
            "beforebegin" => Ok(AdjacentPosition::BeforeBegin),
            "afterbegin"  => Ok(AdjacentPosition::AfterBegin),
            "beforeend"   => Ok(AdjacentPosition::BeforeEnd),
            "afterend"    => Ok(AdjacentPosition::AfterEnd),
            _             => Err(Error::Syntax)
        }
    }
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

    pub fn new_inherited(local_name: Atom,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: &Document) -> Element {
        Element::new_inherited_with_state(ElementState::empty(), local_name,
                                          namespace, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, local_name: Atom,
                                    namespace: Namespace, prefix: Option<DOMString>,
                                    document: &Document)
                                    -> Element {
        Element {
            node: Node::new_inherited(document),
            local_name: local_name,
            tag_name: TagName::new(),
            namespace: namespace,
            prefix: prefix,
            attrs: DOMRefCell::new(vec![]),
            id_attribute: DOMRefCell::new(None),
            style_attribute: DOMRefCell::new(None),
            attr_list: Default::default(),
            class_list: Default::default(),
            state: Cell::new(state),
            atomic_flags: AtomicElementFlags::new(),
        }
    }

    pub fn new(local_name: Atom,
               namespace: Namespace,
               prefix: Option<DOMString>,
               document: &Document) -> Root<Element> {
        Node::reflect_node(
            box Element::new_inherited(local_name, namespace, prefix, document),
            document,
            ElementBinding::Wrap)
    }

    // https://drafts.csswg.org/cssom-view/#css-layout-box
    // Elements that have a computed value of the display property
    // that is table-column or table-column-group
    // FIXME: Currently, it is assumed to be true always
    fn has_css_layout_box(&self) -> bool {
        true
    }

    // https://drafts.csswg.org/cssom-view/#potentially-scrollable
    fn potentially_scrollable(&self) -> bool {
        self.has_css_layout_box() &&
        !self.overflow_x_is_visible() &&
        !self.overflow_y_is_visible()
    }

    // used value of overflow-x is "visible"
    fn overflow_x_is_visible(&self) -> bool {
        let window = window_from_node(self);
        let overflow_pair = window.overflow_query(self.upcast::<Node>().to_trusted_node_address());
        overflow_pair.x == overflow_x::computed_value::T::visible
    }

    // used value of overflow-y is "visible"
    fn overflow_y_is_visible(&self) -> bool {
        let window = window_from_node(self);
        let overflow_pair = window.overflow_query(self.upcast::<Node>().to_trusted_node_address());
        overflow_pair.y != overflow_x::computed_value::T::visible
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
        where V: Push<ApplicableDeclarationBlock>;
    #[allow(unsafe_code)]
    unsafe fn get_colspan(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    fn id_attribute(&self) -> *const Option<Atom>;
    fn style_attribute(&self) -> *const Option<Arc<RwLock<PropertyDeclarationBlock>>>;
    fn local_name(&self) -> &Atom;
    fn namespace(&self) -> &Namespace;
    fn get_checked_state_for_layout(&self) -> bool;
    fn get_indeterminate_state_for_layout(&self) -> bool;
    fn get_state_for_layout(&self) -> ElementState;
    fn insert_atomic_flags(&self, flags: ElementFlags);
}

impl LayoutElementHelpers for LayoutJS<Element> {
    #[allow(unsafe_code)]
    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(), &atom!("class")).map_or(false, |attr| {
            attr.value_tokens_forever().unwrap().iter().any(|atom| atom == name)
        })
    }

    #[allow(unsafe_code)]
    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(), &atom!("class"))
            .map(|attr| attr.value_tokens_forever().unwrap())
    }

    #[allow(unsafe_code)]
    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>
    {
        #[inline]
        fn from_declaration(rule: PropertyDeclaration) -> ApplicableDeclarationBlock {
            ApplicableDeclarationBlock::from_declarations(
                Arc::new(RwLock::new(PropertyDeclarationBlock {
                    declarations: vec![(rule, Importance::Normal)],
                    important_count: 0,
                })),
                Importance::Normal)
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
                    background_image::SpecifiedValue(vec![
                        background_image::single_value::SpecifiedValue(Some(
                            specified::Image::Url(url, specified::UrlExtraData { })
                        ))
                    ])))));
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
                            font_family::computed_value::FontFamily::from_atom(
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
            match (*self.unsafe_get()).get_attr_val_for_layout(&ns!(), &atom!("type")) {
                // Not text entry widget
                Some("hidden") | Some("date") | Some("month") | Some("week") |
                Some("time") | Some("datetime-local") | Some("number") | Some("range") |
                Some("color") | Some("checkbox") | Some("radio") | Some("file") |
                Some("submit") | Some("image") | Some("reset") | Some("button") => {
                    None
                },
                // Others
                _ => {
                    match this.size_for_layout() {
                        0 => None,
                        s => Some(s as i32),
                    }
                },
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
        } else if let Some(this) = self.downcast::<HTMLImageElement>() {
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
        } else if let Some(this) = self.downcast::<HTMLImageElement>() {
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
            match this.get_cols() {
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
            match this.get_rows() {
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
            let width_value = specified::BorderWidth::from_length(
                specified::Length::Absolute(Au::from_px(border as i32)));
            hints.push(from_declaration(
                PropertyDeclaration::BorderTopWidth(DeclaredValue::Value(width_value))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderLeftWidth(DeclaredValue::Value(width_value))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderBottomWidth(DeclaredValue::Value(width_value))));
            hints.push(from_declaration(
                PropertyDeclaration::BorderRightWidth(DeclaredValue::Value(width_value))));
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
        if (*self.unsafe_get()).namespace != ns!(html) {
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
    fn style_attribute(&self) -> *const Option<Arc<RwLock<PropertyDeclarationBlock>>> {
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
                input.checked_state_for_layout()
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
                input.indeterminate_state_for_layout()
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

    #[inline]
    #[allow(unsafe_code)]
    fn insert_atomic_flags(&self, flags: ElementFlags) {
        unsafe {
            (*self.unsafe_get()).atomic_flags.insert(flags);
        }
    }
}

impl Element {
    pub fn html_element_in_html_document(&self) -> bool {
        self.namespace == ns!(html) && self.upcast::<Node>().is_in_html_doc()
    }

    pub fn local_name(&self) -> &Atom {
        &self.local_name
    }

    pub fn parsed_name(&self, mut name: DOMString) -> Atom {
        if self.html_element_in_html_document() {
            name.make_ascii_lowercase();
        }
        Atom::from(name)
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

    pub fn style_attribute(&self) -> &DOMRefCell<Option<Arc<RwLock<PropertyDeclarationBlock>>>> {
        &self.style_attribute
    }

    pub fn summarize(&self) -> Vec<AttrInfo> {
        self.attrs.borrow().iter()
                           .map(|attr| attr.summarize())
                           .collect()
    }

    pub fn is_void(&self) -> bool {
        if self.namespace != ns!(html) {
            return false
        }
        match self.local_name {
            /* List of void elements from
            https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm */

            atom!("area") | atom!("base") | atom!("basefont") | atom!("bgsound") | atom!("br") |
            atom!("col") |  atom!("embed") | atom!("frame") | atom!("hr") | atom!("img") |
            atom!("input") | atom!("keygen") | atom!("link") | atom!("menuitem") | atom!("meta") |
            atom!("param") | atom!("source") | atom!("track") | atom!("wbr") => true,
            _ => false
        }
    }

    // this sync method is called upon modification of the style_attribute property,
    // therefore, it should not trigger subsequent mutation events
    pub fn set_style_attr(&self, new_value: String) {
        let mut new_style = AttrValue::String(new_value);

        if let Some(style_attr) = self.attrs.borrow().iter().find(|a| a.name() == &atom!("style")) {
            style_attr.swap_value(&mut new_style);
            return;
        }

        // explicitly not calling the push_new_attribute convenience method
        // in order to avoid triggering mutation events
        let window = window_from_node(self);
        let attr = Attr::new(&window,
                             atom!("style"),
                             new_style,
                             atom!("style"),
                             ns!(),
                             Some(atom!("style")),
                             Some(self));

         assert!(attr.GetOwnerElement().r() == Some(self));
         self.attrs.borrow_mut().push(JS::from_ref(&attr));
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

    pub fn root_element(&self) -> Root<Element> {
        if self.node.is_in_doc() {
            self.upcast::<Node>()
                .owner_doc()
                .GetDocumentElement()
                .unwrap()
        } else {
            self.upcast::<Node>()
                .inclusive_ancestors()
                .filter_map(Root::downcast)
                .last()
                .expect("We know inclusive_ancestors will return `self` which is an element")
        }
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
                self.disabled_state()
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
        let window = window_from_node(self);
        let attr = Attr::new(&window,
                             local_name,
                             value,
                             name,
                             namespace,
                             prefix,
                             Some(self));
        self.push_attribute(&attr);
    }

    pub fn push_attribute(&self, attr: &Attr) {
        assert!(attr.GetOwnerElement().r() == Some(self));
        self.will_mutate_attr();
        self.attrs.borrow_mut().push(JS::from_ref(attr));
        if attr.namespace() == &ns!() {
            vtable_for(self.upcast()).attribute_mutated(attr, AttributeMutation::Set(None));
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
                Atom::from(name)
            },
        };
        let value = self.parse_attribute(&qname.ns, &qname.local, value);
        self.push_new_attribute(qname.local, value, name, qname.ns, prefix);
    }

    pub fn set_attribute(&self, name: &Atom, value: AttrValue) {
        assert!(name == &name.to_ascii_lowercase());
        assert!(!name.contains(":"));

        self.set_first_matching_attribute(name.clone(),
                                          value,
                                          name.clone(),
                                          ns!(),
                                          None,
                                          |attr| attr.local_name() == name);
    }

    // https://html.spec.whatwg.org/multipage/#attr-data-*
    pub fn set_custom_attribute(&self, name: DOMString, value: DOMString) -> ErrorResult {
        // Step 1.
        if let InvalidXMLName = xml_name_type(&name) {
            return Err(Error::InvalidCharacter);
        }

        // Steps 2-5.
        let name = Atom::from(name);
        let value = self.parse_attribute(&ns!(), &name, value);
        self.set_first_matching_attribute(name.clone(),
                                          value,
                                          name.clone(),
                                          ns!(),
                                          None,
                                          |attr| {
                                              *attr.name() == name && *attr.namespace() == ns!()
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
        if *namespace == ns!() {
            vtable_for(self.upcast()).parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value.into())
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
            if attr.namespace() == &ns!() {
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
        self.get_attribute(&ns!(), &atom!("class"))
            .map_or(false, |attr| attr.value().as_tokens().iter().any(|atom| is_equal(name, atom)))
    }

    pub fn set_atomic_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value.into());
        self.set_attribute(local_name, value);
    }

    pub fn has_attribute(&self, local_name: &Atom) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        self.attrs
            .borrow()
            .iter()
            .any(|attr| attr.local_name() == local_name && attr.namespace() == &ns!())
    }

    pub fn set_bool_attribute(&self, local_name: &Atom, value: bool) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(local_name, DOMString::new());
        } else {
            self.remove_attribute(&ns!(), local_name);
        }
    }

    pub fn get_url_attribute(&self, local_name: &Atom) -> DOMString {
        assert!(*local_name == local_name.to_ascii_lowercase());
        if !self.has_attribute(local_name) {
            return DOMString::new();
        }
        let url = self.get_string_attribute(local_name);
        let doc = document_from_node(self);
        let base = doc.base_url();
        // https://html.spec.whatwg.org/multipage/#reflect
        // XXXManishearth this doesn't handle `javascript:` urls properly
        match base.join(&url) {
            Ok(parsed) => DOMString::from(parsed.into_string()),
            Err(_) => DOMString::from(""),
        }
    }
    pub fn set_url_attribute(&self, local_name: &Atom, value: DOMString) {
        self.set_string_attribute(local_name, value);
    }

    pub fn get_string_attribute(&self, local_name: &Atom) -> DOMString {
        match self.get_attribute(&ns!(), local_name) {
            Some(x) => x.Value(),
            None => DOMString::new(),
        }
    }
    pub fn set_string_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value.into()));
    }

    pub fn get_tokenlist_attribute(&self, local_name: &Atom) -> Vec<Atom> {
        self.get_attribute(&ns!(), local_name).map(|attr| {
            attr.value()
                .as_tokens()
                .to_vec()
        }).unwrap_or(vec!())
    }

    pub fn set_tokenlist_attribute(&self, local_name: &Atom, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name,
                           AttrValue::from_serialized_tokenlist(value.into()));
    }

    pub fn set_atomic_tokenlist_attribute(&self, local_name: &Atom, tokens: Vec<Atom>) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens));
    }

    pub fn get_int_attribute(&self, local_name: &Atom, default: i32) -> i32 {
        // TODO: Is this assert necessary?
        assert!(local_name.chars().all(|ch| {
            !ch.is_ascii() || ch.to_ascii_lowercase() == ch
        }));
        let attribute = self.get_attribute(&ns!(), local_name);

        match attribute {
            Some(ref attribute) => {
                match *attribute.value() {
                    AttrValue::Int(_, value) => value,
                    _ => panic!("Expected an AttrValue::Int: \
                                 implement parse_plain_attribute"),
                }
            }
            None => default,
        }
    }

    pub fn set_int_attribute(&self, local_name: &Atom, value: i32) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::Int(value.to_string(), value));
    }

    pub fn get_uint_attribute(&self, local_name: &Atom, default: u32) -> u32 {
        assert!(local_name.chars().all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch));
        let attribute = self.get_attribute(&ns!(), local_name);
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
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::UInt(value.to_string(), value));
    }

    pub fn will_mutate_attr(&self) {
        let node = self.upcast::<Node>();
        node.owner_doc().element_attr_will_change(self);
    }

    // https://dom.spec.whatwg.org/#insert-adjacent
    pub fn insert_adjacent(&self, where_: AdjacentPosition, node: &Node)
                           -> Fallible<Option<Root<Node>>> {
        let self_node = self.upcast::<Node>();
        match where_ {
            AdjacentPosition::BeforeBegin => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, Some(self_node)).map(Some)
                } else {
                    Ok(None)
                }
            }
            AdjacentPosition::AfterBegin => {
                Node::pre_insert(node, &self_node, self_node.GetFirstChild().r()).map(Some)
            }
            AdjacentPosition::BeforeEnd => {
                Node::pre_insert(node, &self_node, None).map(Some)
            }
            AdjacentPosition::AfterEnd => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, self_node.GetNextSibling().r()).map(Some)
                } else {
                    Ok(None)
                }
            }
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    pub fn scroll(&self, x_: f64, y_: f64, behavior: ScrollBehavior) {
        // Step 1.2 or 2.3
        let x = if x_.is_finite() { x_ } else { 0.0f64 };
        let y = if y_.is_finite() { y_ } else { 0.0f64 };

        let node = self.upcast::<Node>();

        // Step 3
        let doc = node.owner_doc();

        // Step 4
        if !doc.is_fully_active() {
            return;
        }

        // Step 5
        let win = match doc.GetDefaultView() {
            None => return,
            Some(win) => win,
        };

        // Step 7
        if *self.root_element() == *self {
            if doc.quirks_mode() != Quirks {
                win.scroll(x, y, behavior);
            }

            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == Quirks &&
           !self.potentially_scrollable() {
               win.scroll(x, y, behavior);
               return;
        }

        // Step 10 (TODO)

        // Step 11
        win.scroll_node(node.to_trusted_node_address(), x, y, behavior);
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
        let name = self.tag_name.or_init(|| {
            let qualified_name = match self.prefix {
                Some(ref prefix) => {
                    Cow::Owned(format!("{}:{}", &**prefix, &*self.local_name))
                },
                None => Cow::Borrowed(&*self.local_name)
            };
            if self.html_element_in_html_document() {
                Atom::from(qualified_name.to_ascii_uppercase())
            } else {
                Atom::from(qualified_name)
            }
        });
        DOMString::from(&*name)
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

    // https://dom.spec.whatwg.org/#dom-element-hasattributes
    fn HasAttributes(&self) -> bool {
        !self.attrs.borrow().is_empty()
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributenames
    fn GetAttributeNames(&self) -> Vec<DOMString> {
        self.attrs.borrow().iter().map(|attr| attr.Name()).collect()
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
        self.get_attribute(namespace, &Atom::from(local_name))
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
        let value = self.parse_attribute(&ns!(), &name, value);
        self.set_first_matching_attribute(
            name.clone(), value, name.clone(), ns!(), None,
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
        let qualified_name = Atom::from(qualified_name);
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.set_first_matching_attribute(
            local_name.clone(), value, qualified_name, namespace.clone(), prefix,
            |attr| *attr.local_name() == local_name && *attr.namespace() == namespace);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributenode
    fn SetAttributeNode(&self, attr: &Attr) -> Fallible<Option<Root<Attr>>> {
        // Step 1.
        if let Some(owner) = attr.GetOwnerElement() {
            if &*owner != self {
                return Err(Error::InUseAttribute);
            }
        }

        // Step 2.
        let position = self.attrs.borrow().iter().position(|old_attr| {
            attr.namespace() == old_attr.namespace() && attr.local_name() == old_attr.local_name()
        });

        if let Some(position) = position {
            let old_attr = Root::from_ref(&*self.attrs.borrow()[position]);

            // Step 3.
            if &*old_attr == attr {
                return Ok(Some(Root::from_ref(attr)));
            }

            // Step 4.
            self.will_mutate_attr();
            attr.set_owner(Some(self));
            self.attrs.borrow_mut()[position] = JS::from_ref(attr);
            old_attr.set_owner(None);
            if attr.namespace() == &ns!() {
                vtable_for(self.upcast()).attribute_mutated(
                    &attr, AttributeMutation::Set(Some(&old_attr.value())));
            }

            // Step 6.
            Ok(Some(old_attr))
        } else {
            // Step 5.
            attr.set_owner(Some(self));
            self.push_attribute(attr);

            // Step 6.
            Ok(None)
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributenodens
    fn SetAttributeNodeNS(&self, attr: &Attr) -> Fallible<Option<Root<Attr>>> {
        self.SetAttributeNode(attr)
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(&self, name: DOMString) {
        let name = self.parsed_name(name);
        self.remove_attribute_by_name(&name);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) {
        let namespace = namespace_from_domstring(namespace);
        let local_name = Atom::from(local_name);
        self.remove_attribute(&namespace, &local_name);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributenode
    fn RemoveAttributeNode(&self, attr: &Attr) -> Fallible<Root<Attr>> {
        self.remove_first_matching_attribute(|a| a == attr)
            .ok_or(Error::NotFound)
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
        HTMLCollection::by_tag_name(&window, self.upcast(), localname)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(&self,
                              maybe_ns: Option<DOMString>,
                              localname: DOMString)
                              -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_tag_name_ns(&window, self.upcast(), localname, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> Root<HTMLCollection> {
        let window = window_from_node(self);
        HTMLCollection::by_class_name(&window, self.upcast(), classes)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self) -> Root<DOMRectList> {
        let win = window_from_node(self);
        let raw_rects = self.upcast::<Node>().content_boxes();
        let rects = raw_rects.iter().map(|rect| {
            DOMRect::new(win.upcast(),
                         rect.origin.x.to_f64_px(),
                         rect.origin.y.to_f64_px(),
                         rect.size.width.to_f64_px(),
                         rect.size.height.to_f64_px())
        });
        DOMRectList::new(&win, rects)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self) -> Root<DOMRect> {
        let win = window_from_node(self);
        let rect = self.upcast::<Node>().bounding_content_box();
        DOMRect::new(win.upcast(),
                     rect.origin.x.to_f64_px(),
                     rect.origin.y.to_f64_px(),
                     rect.size.width.to_f64_px(),
                     rect.size.height.to_f64_px())
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    fn Scroll(&self, options: &ScrollToOptions) {
        // Step 1
        let left = options.left.unwrap_or(self.ScrollLeft());
        let top = options.top.unwrap_or(self.ScrollTop());
        self.scroll(left, top, options.parent.behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    fn Scroll_(&self, x: f64, y: f64) {
        self.scroll(x, y, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollto
    fn ScrollTo(&self, options: &ScrollToOptions) {
        self.Scroll(options);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollto
    fn ScrollTo_(&self, x: f64, y: f64) {
        self.Scroll_(x, y);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollby
    fn ScrollBy(&self, options: &ScrollToOptions) {
        // Step 2
        let delta_left = options.left.unwrap_or(0.0f64);
        let delta_top = options.top.unwrap_or(0.0f64);
        let left = self.ScrollLeft();
        let top = self.ScrollTop();
        self.scroll(left + delta_left, top + delta_top,
                    options.parent.behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollby
    fn ScrollBy_(&self, x: f64, y: f64) {
        let left = self.ScrollLeft();
        let top = self.ScrollTop();
        self.scroll(left + x, top + y, ScrollBehavior::Auto);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn ScrollTop(&self) -> f64 {
        let node = self.upcast::<Node>();

        // Step 1
        let doc = node.owner_doc();

        // Step 2
        if !doc.is_fully_active() {
            return 0.0;
        }

        // Step 3
        let win = match doc.GetDefaultView() {
            None => return 0.0,
            Some(win) => win,
        };

        // Step 5
        if *self.root_element() == *self {
            if doc.quirks_mode() == Quirks {
                return 0.0;
            }

            // Step 6
            return win.ScrollY() as f64;
        }

        // Step 7
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == Quirks &&
           !self.potentially_scrollable() {
               return win.ScrollY() as f64;
        }


        // Step 8
        if !self.has_css_layout_box() {
            return 0.0;
        }

        // Step 9
        let point = node.scroll_offset();
        return point.y.abs() as f64;
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn SetScrollTop(&self, y_: f64) {
        let behavior = ScrollBehavior::Auto;

        // Step 1, 2
        let y = if y_.is_finite() { y_ } else { 0.0f64 };

        let node = self.upcast::<Node>();

        // Step 3
        let doc = node.owner_doc();

        // Step 4
        if !doc.is_fully_active() {
            return;
        }

        // Step 5
        let win = match doc.GetDefaultView() {
            None => return,
            Some(win) => win,
        };

        // Step 7
        if *self.root_element() == *self {
            if doc.quirks_mode() != Quirks {
                win.scroll(win.ScrollX() as f64, y, behavior);
            }

            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == Quirks &&
           !self.potentially_scrollable() {
               win.scroll(win.ScrollX() as f64, y, behavior);
               return;
        }

        // Step 10 (TODO)

        // Step 11
        win.scroll_node(node.to_trusted_node_address(), self.ScrollLeft(), y, behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn ScrollLeft(&self) -> f64 {
        let node = self.upcast::<Node>();

        // Step 1
        let doc = node.owner_doc();

        // Step 2
        if !doc.is_fully_active() {
            return 0.0;
        }

        // Step 3
        let win = match doc.GetDefaultView() {
            None => return 0.0,
            Some(win) => win,
        };

        // Step 5
        if *self.root_element() == *self {
            if doc.quirks_mode() != Quirks {
                // Step 6
                return win.ScrollX() as f64;
            }

            return 0.0;
        }

        // Step 7
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == Quirks &&
           !self.potentially_scrollable() {
               return win.ScrollX() as f64;
        }


        // Step 8
        if !self.has_css_layout_box() {
            return 0.0;
        }

        // Step 9
        let point = node.scroll_offset();
        return point.x.abs() as f64;
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollleft
    fn SetScrollLeft(&self, x_: f64) {
        let behavior = ScrollBehavior::Auto;

        // Step 1, 2
        let x = if x_.is_finite() { x_ } else { 0.0f64 };

        let node = self.upcast::<Node>();

        // Step 3
        let doc = node.owner_doc();

        // Step 4
        if !doc.is_fully_active() {
            return;
        }

        // Step 5
        let win = match doc.GetDefaultView() {
            None => return,
            Some(win) => win,
        };

        // Step 7
        if *self.root_element() == *self {
            if doc.quirks_mode() == Quirks {
                return;
            }

            win.scroll(x, win.ScrollY() as f64, behavior);
            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == Quirks &&
           !self.potentially_scrollable() {
               win.scroll(x, win.ScrollY() as f64, behavior);
               return;
        }

        // Step 10 (TODO)

        // Step 11
        win.scroll_node(node.to_trusted_node_address(), x, self.ScrollTop(), behavior);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth
    fn ScrollWidth(&self) -> i32 {
        self.upcast::<Node>().scroll_area().size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollheight
    fn ScrollHeight(&self) -> i32 {
        self.upcast::<Node>().scroll_area().size.height
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
    fn ClientTop(&self) -> i32 {
        self.upcast::<Node>().client_rect().origin.y
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientleft
    fn ClientLeft(&self) -> i32 {
        self.upcast::<Node>().client_rect().origin.x
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientwidth
    fn ClientWidth(&self) -> i32 {
        self.upcast::<Node>().client_rect().size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientheight
    fn ClientHeight(&self) -> i32 {
        self.upcast::<Node>().client_rect().size.height
    }

    /// https://w3c.github.io/DOM-Parsing/#widl-Element-innerHTML
    fn GetInnerHTML(&self) -> Fallible<DOMString> {
        // XXX TODO: XML case
        self.serialize(ChildrenOnly)
    }

    /// https://w3c.github.io/DOM-Parsing/#widl-Element-innerHTML
    fn SetInnerHTML(&self, value: DOMString) -> ErrorResult {
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
    fn SetOuterHTML(&self, value: DOMString) -> ErrorResult {
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
            NodeTypeId::Document(_) => return Err(Error::NoModificationAllowed),

            // Step 4.
            NodeTypeId::DocumentFragment => {
                let body_elem = Element::create(QualName::new(ns!(html), atom!("body")),
                                                None, &context_document,
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
        HTMLCollection::children(&window, self.upcast())
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
                Ok(matches(selectors, &Root::from_ref(self), None, MatchingReason::Other))
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
                        if matches(selectors, &element, None, MatchingReason::Other) {
                            return Ok(Some(element));
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacentelement
    fn InsertAdjacentElement(&self, where_: DOMString, element: &Element)
                             -> Fallible<Option<Root<Element>>> {
        let where_ = try!(AdjacentPosition::try_from(&*where_));
        let inserted_node = try!(self.insert_adjacent(where_, element.upcast()));
        Ok(inserted_node.map(|node| Root::downcast(node).unwrap()))
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacenttext
    fn InsertAdjacentText(&self, where_: DOMString, data: DOMString)
                          -> ErrorResult {
        // Step 1.
        let text = Text::new(data, &document_from_node(self));

        // Step 2.
        let where_ = try!(AdjacentPosition::try_from(&*where_));
        self.insert_adjacent(where_, text.upcast()).map(|_| ())
    }

    // https://w3c.github.io/DOM-Parsing/#dom-element-insertadjacenthtml
    fn InsertAdjacentHTML(&self, position: DOMString, text: DOMString)
                          -> ErrorResult {
        // Step 1.
        let position = try!(AdjacentPosition::try_from(&*position));

        let context = match position {
            AdjacentPosition::BeforeBegin | AdjacentPosition::AfterEnd => {
                match self.upcast::<Node>().GetParentNode() {
                    Some(ref node) if node.is::<Document>() => {
                        return Err(Error::NoModificationAllowed)
                    }
                    None => return Err(Error::NoModificationAllowed),
                    Some(node) => node,
                }
            }
            AdjacentPosition::AfterBegin | AdjacentPosition::BeforeEnd => {
                Root::from_ref(self.upcast::<Node>())
            }
        };

        // Step 2.
        let context = match context.downcast::<Element>() {
            Some(elem) if elem.local_name() != &atom!("html") ||
                          !elem.html_element_in_html_document() => Root::from_ref(elem),
            _ => Root::upcast(HTMLBodyElement::new(atom!("body"), None, &*context.owner_doc())),
        };

        // Step 3.
        let fragment = try!(context.upcast::<Node>().parse_fragment(text));

        // Step 4.
        self.insert_adjacent(position, fragment.upcast()).map(|_| ())
    }

    // check-tidy: no specs after this line
    fn EnterFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.enter_formal_activation_state();
                return Ok(());
            },
            None => return Err(Error::NotSupported)
        }
    }

    fn ExitFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.exit_formal_activation_state();
                return Ok(());
            },
            None => return Err(Error::NotSupported)
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
            &atom!("style") => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() =
                    mutation.new_value(attr).map(|value| {
                        let win = window_from_node(self);
                        Arc::new(RwLock::new(parse_style_attribute(
                            &value,
                            &doc.base_url(),
                            win.css_error_reporter(),
                            ParserContextExtraData::default())))
                    });
                if node.is_in_doc() {
                    node.dirty(NodeDamage::NodeStyleDamaged);
                }
            },
            &atom!("id") => {
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
            _ if attr.namespace() == &ns!() => {
                if fragment_affecting_attributes().iter().any(|a| a == attr.local_name()) ||
                   common_style_affecting_attributes().iter().any(|a| &a.atom == attr.local_name()) ||
                   rare_style_affecting_attributes().iter().any(|a| a == attr.local_name())
                {
                    node.dirty(NodeDamage::OtherNodeDamage);
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
            &atom!("id") => AttrValue::from_atomic(value.into()),
            &atom!("class") => AttrValue::from_serialized_tokenlist(value.into()),
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

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if !context.tree_in_doc {
            return;
        }

        if let Some(ref value) = *self.id_attribute.borrow() {
            let doc = document_from_node(self);
            doc.unregister_named_element(self, value.clone());
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }

        let flags = self.atomic_flags.get();
        if flags.intersects(HAS_SLOW_SELECTOR) {
            // All children of this node need to be restyled when any child changes.
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        } else {
            if flags.intersects(HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
                if let Some(next_child) = mutation.next_child() {
                    for child in next_child.inclusively_following_siblings() {
                        if child.is::<Element>() {
                            child.dirty(NodeDamage::OtherNodeDamage);
                        }
                    }
                }
            }
            if flags.intersects(HAS_EDGE_CHILD_SELECTOR) {
                if let Some(child) = mutation.modified_edge_element() {
                    child.dirty(NodeDamage::OtherNodeDamage);
                }
            }
        }
    }

    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);

        if document_from_node(self).is_html_document() != old_doc.is_html_document() {
            self.tag_name.clear();
        }
    }
}

impl<'a> ::selectors::MatchAttrGeneric for Root<Element> {
    type Impl = ServoSelectorImpl;

    fn match_attr<F>(&self, attr: &AttrSelector<ServoSelectorImpl>, test: F) -> bool
        where F: Fn(&str) -> bool
    {
        use ::selectors::Element;
        let local_name = {
            if self.is_html_element_in_html_document() {
                &attr.lower_name
            } else {
                &attr.name
            }
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attribute(&ns.url, local_name)
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

    fn get_local_name(&self) -> &Atom {
        self.local_name()
    }

    fn get_namespace(&self) -> &Namespace {
        self.namespace()
    }

    fn match_non_ts_pseudo_class(&self, pseudo_class: NonTSPseudoClass) -> bool {
        match pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link |
            NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::ServoNonZeroBorder => {
                match self.downcast::<HTMLTableElement>() {
                    None => false,
                    Some(this) => {
                        match this.get_border() {
                            None | Some(0) => false,
                            Some(_) => true,
                        }
                    }
                }
            },

            NonTSPseudoClass::ReadOnly =>
                !Element::state(self).contains(pseudo_class.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::Target =>
                Element::state(self).contains(pseudo_class.state_flag()),
        }
    }

    fn get_id(&self) -> Option<Atom> {
        self.id_attribute.borrow().clone()
    }

    fn has_class(&self, name: &Atom) -> bool {
        Element::has_class(&**self, name)
    }

    fn each_class<F>(&self, mut callback: F)
        where F: FnMut(&Atom)
    {
        if let Some(ref attr) = self.get_attribute(&ns!(), &atom!("class")) {
            let tokens = attr.value();
            let tokens = tokens.as_tokens();
            for token in tokens {
                callback(token);
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
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
                let element = self.downcast::<HTMLButtonElement>().unwrap();
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

    // https://html.spec.whatwg.org/multipage/#category-submit
    pub fn as_maybe_validatable(&self) -> Option<&Validatable> {
        let element = match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) => {
                let element = self.downcast::<HTMLInputElement>().unwrap();
                Some(element as &Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) => {
                let element = self.downcast::<HTMLButtonElement>().unwrap();
                Some(element as &Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement)) => {
                let element = self.downcast::<HTMLObjectElement>().unwrap();
                Some(element as &Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) => {
                let element = self.downcast::<HTMLSelectElement>().unwrap();
                Some(element as &Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                let element = self.downcast::<HTMLTextAreaElement>().unwrap();
                Some(element as &Validatable)
            },
            _ => {
                None
            }
        };
        element
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

    pub fn state(&self) -> ElementState {
        self.state.get()
    }

    pub fn set_state(&self, which: ElementState, value: bool) {
        let mut state = self.state.get();
        if state.contains(which) == value {
            return;
        }
        let node = self.upcast::<Node>();
        node.owner_doc().element_state_will_change(self);
        if value {
            state.insert(which);
        } else {
            state.remove(which);
        }
        self.state.set(state);
    }

    pub fn active_state(&self) -> bool {
        self.state.get().contains(IN_ACTIVE_STATE)
    }

    /// https://html.spec.whatwg.org/multipage/#concept-selector-active
    pub fn set_active_state(&self, value: bool) {
        self.set_state(IN_ACTIVE_STATE, value);

        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            parent.set_active_state(value);
        }
    }

    pub fn focus_state(&self) -> bool {
        self.state.get().contains(IN_FOCUS_STATE)
    }

    pub fn set_focus_state(&self, value: bool) {
        self.set_state(IN_FOCUS_STATE, value);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    pub fn hover_state(&self) -> bool {
        self.state.get().contains(IN_HOVER_STATE)
    }

    pub fn set_hover_state(&self, value: bool) {
        self.set_state(IN_HOVER_STATE, value)
    }

    pub fn enabled_state(&self) -> bool {
        self.state.get().contains(IN_ENABLED_STATE)
    }

    pub fn set_enabled_state(&self, value: bool) {
        self.set_state(IN_ENABLED_STATE, value)
    }

    pub fn disabled_state(&self) -> bool {
        self.state.get().contains(IN_DISABLED_STATE)
    }

    pub fn set_disabled_state(&self, value: bool) {
        self.set_state(IN_DISABLED_STATE, value)
    }

    pub fn read_write_state(&self) -> bool {
        self.state.get().contains(IN_READ_WRITE_STATE)
    }

    pub fn set_read_write_state(&self, value: bool) {
        self.set_state(IN_READ_WRITE_STATE, value)
    }

    pub fn placeholder_shown_state(&self) -> bool {
        self.state.get().contains(IN_PLACEHOLDER_SHOWN_STATE)
    }

    pub fn set_placeholder_shown_state(&self, value: bool) {
        if self.placeholder_shown_state() != value {
            self.set_state(IN_PLACEHOLDER_SHOWN_STATE, value);
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        }
    }

    pub fn target_state(&self) -> bool {
        self.state.get().contains(IN_TARGET_STATE)
    }

    pub fn set_target_state(&self, value: bool) {
       self.set_state(IN_TARGET_STATE, value)
    }
}

impl Element {
    pub fn check_ancestors_disabled_state_for_form_control(&self) {
        let node = self.upcast::<Node>();
        if self.disabled_state() {
            return;
        }
        for ancestor in node.ancestors() {
            if !ancestor.is::<HTMLFieldSetElement>() {
                continue;
            }
            if !ancestor.downcast::<Element>().unwrap().disabled_state() {
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
        if self.disabled_state() {
            return;
        }
        let node = self.upcast::<Node>();
        if let Some(ref parent) = node.GetParentNode() {
            if parent.is::<HTMLOptGroupElement>() &&
               parent.downcast::<Element>().unwrap().disabled_state() {
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

/// Thread-safe wrapper for ElementFlags set during selector matching
#[derive(JSTraceable, HeapSizeOf)]
struct AtomicElementFlags(AtomicUsize);

impl AtomicElementFlags {
    fn new() -> Self {
        AtomicElementFlags(AtomicUsize::new(0))
    }

    fn get(&self) -> ElementFlags {
        ElementFlags::from_bits_truncate(self.0.load(Ordering::Relaxed) as u8)
    }

    fn insert(&self, flags: ElementFlags) {
        self.0.fetch_or(flags.bits() as usize, Ordering::Relaxed);
    }
}

/// A holder for an element's "tag name", which will be lazily
/// resolved and cached. Should be reset when the document
/// owner changes.
#[derive(JSTraceable, HeapSizeOf)]
struct TagName {
    ptr: DOMRefCell<Option<Atom>>,
}

impl TagName {
    fn new() -> TagName {
        TagName { ptr: DOMRefCell::new(None) }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    fn or_init<F>(&self, cb: F) -> Atom
        where F: FnOnce() -> Atom
    {
        match &mut *self.ptr.borrow_mut() {
            &mut Some(ref name) => name.clone(),
            ptr => {
                let name = cb();
                *ptr = Some(name.clone());
                name
            }
        }
    }

    /// Clear the cached tag name, so that it will be re-calculated the
    /// next time that `or_init()` is called.
    fn clear(&self) {
        *self.ptr.borrow_mut() = None;
    }
}
