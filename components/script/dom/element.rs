/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Element nodes.

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
use dom::bindings::conversions::DerivedFrom;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::{JS, LayoutJS, MutNullableJS};
use dom::bindings::js::{Root, RootedReference};
use dom::bindings::refcounted::{Trusted, TrustedPromise};
use dom::bindings::reflector::DomObject;
use dom::bindings::str::{DOMString, extended_filtering};
use dom::bindings::xmlname::{namespace_from_domstring, validate_and_extract, xml_name_type};
use dom::bindings::xmlname::XMLName::InvalidXMLName;
use dom::characterdata::CharacterData;
use dom::create::create_element;
use dom::document::{Document, LayoutDocumentHelpers};
use dom::documentfragment::DocumentFragment;
use dom::domrect::DOMRect;
use dom::domrectlist::DOMRectList;
use dom::domtokenlist::DOMTokenList;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlanchorelement::HTMLAnchorElement;
use dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementLayoutHelpers};
use dom::htmlbuttonelement::HTMLButtonElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlfontelement::{HTMLFontElement, HTMLFontElementLayoutHelpers};
use dom::htmlformelement::FormControlElementHelpers;
use dom::htmlhrelement::{HTMLHRElement, HTMLHRLayoutHelpers};
use dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use dom::htmllabelelement::HTMLLabelElement;
use dom::htmllegendelement::HTMLLegendElement;
use dom::htmllinkelement::HTMLLinkElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::htmlstyleelement::HTMLStyleElement;
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
use dom::promise::Promise;
use dom::servoparser::ServoParser;
use dom::text::Text;
use dom::validation::Validatable;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::ReflowReason;
use dom_struct::dom_struct;
use html5ever::serialize;
use html5ever::serialize::SerializeOpts;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever_atoms::{Prefix, LocalName, Namespace, QualName};
use js::jsapi::{HandleValue, JSAutoCompartment};
use net_traits::request::CorsSettings;
use ref_filter_map::ref_filter_map;
use script_layout_interface::message::ReflowQueryType;
use script_thread::Runnable;
use selectors::matching::{ElementSelectorFlags, StyleRelations, matches};
use selectors::matching::{HAS_EDGE_CHILD_SELECTOR, HAS_SLOW_SELECTOR, HAS_SLOW_SELECTOR_LATER_SIBLINGS};
use selectors::parser::{AttrSelector, NamespaceConstraint};
use servo_atoms::Atom;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::{Cell, Ref};
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::context::{QuirksMode, ReflowGoal};
use style::element_state::*;
use style::matching::{common_style_affecting_attributes, rare_style_affecting_attributes};
use style::parser::ParserContextExtraData;
use style::properties::{Importance, PropertyDeclaration, PropertyDeclarationBlock, parse_style_attribute};
use style::properties::longhands::{self, background_image, border_spacing, font_family, font_size, overflow_x};
use style::restyle_hints::RESTYLE_SELF;
use style::rule_tree::CascadeLevel;
use style::selector_parser::{NonTSPseudoClass, RestyleDamage, SelectorImpl, SelectorParser};
use style::shared_lock::{SharedRwLock, Locked};
use style::sink::Push;
use style::stylist::ApplicableDeclarationBlock;
use style::thread_state;
use style::values::CSSFloat;
use style::values::specified::{self, CSSColor};
use stylesheet_loader::StylesheetOwner;

// TODO: Update focus state when the top-level browsing context gains or loses system focus,
// and when the element enters or leaves a browsing context container.
// https://html.spec.whatwg.org/multipage/#selector-focus

#[dom_struct]
pub struct Element {
    node: Node,
    local_name: LocalName,
    tag_name: TagName,
    namespace: Namespace,
    prefix: Option<DOMString>,
    attrs: DOMRefCell<Vec<JS<Attr>>>,
    id_attribute: DOMRefCell<Option<Atom>>,
    #[ignore_heap_size_of = "Arc"]
    style_attribute: DOMRefCell<Option<Arc<Locked<PropertyDeclarationBlock>>>>,
    attr_list: MutNullableJS<NamedNodeMap>,
    class_list: MutNullableJS<DOMTokenList>,
    state: Cell<ElementState>,
    /// These flags are set by the style system to indicate the that certain
    /// operations may require restyling this element or its descendants. The
    /// flags are not atomic, so the style system takes care of only set them
    /// when it has exclusive access to the element.
    #[ignore_heap_size_of = "bitflags defined in rust-selectors"]
    selector_flags: Cell<ElementSelectorFlags>,
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
    ParserCreated(u64),
    ScriptCreated,
}

impl ElementCreator {
    pub fn is_parser_created(&self) -> bool {
        match *self {
            ElementCreator::ParserCreated(_) => true,
            ElementCreator::ScriptCreated => false,
        }
    }
    pub fn return_line_number(&self) -> u64 {
        match *self {
            ElementCreator::ParserCreated(l) => l,
            ElementCreator::ScriptCreated => 1,
        }
    }
}

pub enum AdjacentPosition {
    BeforeBegin,
    AfterEnd,
    AfterBegin,
    BeforeEnd,
}

impl<'a> TryFrom<&'a str> for AdjacentPosition {
    type Error = Error;

    fn try_from(position: &'a str) -> Result<AdjacentPosition, Self::Error> {
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
    pub fn create(name: QualName, prefix: Option<Prefix>,
                  document: &Document, creator: ElementCreator)
                  -> Root<Element> {
        create_element(name, prefix, document, creator)
    }

    pub fn new_inherited(local_name: LocalName,
                         namespace: Namespace, prefix: Option<DOMString>,
                         document: &Document) -> Element {
        Element::new_inherited_with_state(ElementState::empty(), local_name,
                                          namespace, prefix, document)
    }

    pub fn new_inherited_with_state(state: ElementState, local_name: LocalName,
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
            selector_flags: Cell::new(ElementSelectorFlags::empty()),
        }
    }

    pub fn new(local_name: LocalName,
               namespace: Namespace,
               prefix: Option<DOMString>,
               document: &Document) -> Root<Element> {
        Node::reflect_node(
            box Element::new_inherited(local_name, namespace, prefix, document),
            document,
            ElementBinding::Wrap)
    }

    pub fn restyle(&self, damage: NodeDamage) {
        let doc = self.node.owner_doc();
        let mut restyle = doc.ensure_pending_restyle(self);

        // FIXME(bholley): I think we should probably only do this for
        // NodeStyleDamaged, but I'm preserving existing behavior.
        restyle.hint |= RESTYLE_SELF;

        if damage == NodeDamage::OtherNodeDamage {
            restyle.damage = RestyleDamage::rebuild_and_reflow();
        }
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
    unsafe fn get_attr_for_layout<'a>(&'a self, namespace: &Namespace, name: &LocalName)
                                      -> Option<&'a AttrValue>;
    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &LocalName)
                                      -> Option<&'a str>;
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &LocalName) -> Vec<&'a str>;
}

#[inline]
#[allow(unsafe_code)]
pub unsafe fn get_attr_for_layout<'a>(elem: &'a Element, namespace: &Namespace, name: &LocalName)
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
    unsafe fn get_attr_for_layout<'a>(&'a self, namespace: &Namespace, name: &LocalName)
                                      -> Option<&'a AttrValue> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            attr.value_forever()
        })
    }

    unsafe fn get_attr_val_for_layout<'a>(&'a self, namespace: &Namespace, name: &LocalName)
                                          -> Option<&'a str> {
        get_attr_for_layout(self, namespace, name).map(|attr| {
            attr.value_ref_forever()
        })
    }

    #[inline]
    unsafe fn get_attr_vals_for_layout<'a>(&'a self, name: &LocalName) -> Vec<&'a str> {
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
    unsafe fn get_rowspan(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn html_element_in_html_document_for_layout(&self) -> bool;
    fn id_attribute(&self) -> *const Option<Atom>;
    fn style_attribute(&self) -> *const Option<Arc<Locked<PropertyDeclarationBlock>>>;
    fn local_name(&self) -> &LocalName;
    fn namespace(&self) -> &Namespace;
    fn get_lang_for_layout(&self) -> String;
    fn get_checked_state_for_layout(&self) -> bool;
    fn get_indeterminate_state_for_layout(&self) -> bool;
    fn get_state_for_layout(&self) -> ElementState;
    fn insert_selector_flags(&self, flags: ElementSelectorFlags);
    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool;
}

impl LayoutElementHelpers for LayoutJS<Element> {
    #[allow(unsafe_code)]
    #[inline]
    unsafe fn has_class_for_layout(&self, name: &Atom) -> bool {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(), &local_name!("class")).map_or(false, |attr| {
            attr.value_tokens_forever().unwrap().iter().any(|atom| atom == name)
        })
    }

    #[allow(unsafe_code)]
    #[inline]
    unsafe fn get_classes_for_layout(&self) -> Option<&'static [Atom]> {
        get_attr_for_layout(&*self.unsafe_get(), &ns!(), &local_name!("class"))
            .map(|attr| attr.value_tokens_forever().unwrap())
    }

    #[allow(unsafe_code)]
    unsafe fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>
    {
        #[inline]
        fn from_declaration(shared_lock: &SharedRwLock, declaration: PropertyDeclaration)
                            -> ApplicableDeclarationBlock {
            ApplicableDeclarationBlock::from_declarations(
                Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
                    declaration, Importance::Normal
                ))),
                CascadeLevel::PresHints)
        }

        let document = self.upcast::<Node>().owner_doc_for_layout();
        let shared_lock = document.style_shared_lock();

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
                shared_lock,
                PropertyDeclaration::BackgroundColor(
                    CSSColor { parsed: Color::RGBA(color), authored: None })));
        }

        let background = if let Some(this) = self.downcast::<HTMLBodyElement>() {
            this.get_background()
        } else {
            None
        };

        if let Some(url) = background {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BackgroundImage(
                    background_image::SpecifiedValue(vec![
                        background_image::single_value::SpecifiedValue(Some(
                            specified::Image::for_cascade(url.into())
                        ))
                    ]))));
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
                shared_lock,
                PropertyDeclaration::Color(
                    longhands::color::SpecifiedValue(CSSColor {
                        parsed: Color::RGBA(color),
                        authored: None,
                    })
                )
            ));
        }

        let font_family = if let Some(this) = self.downcast::<HTMLFontElement>() {
            this.get_face()
        } else {
            None
        };

        if let Some(font_family) = font_family {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::FontFamily(
                        font_family::computed_value::T(vec![
                            font_family::computed_value::FontFamily::from_atom(
                                font_family)]))));
        }

        let font_size = self.downcast::<HTMLFontElement>().and_then(|this| this.get_size());

        if let Some(font_size) = font_size {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::FontSize(
                    font_size::SpecifiedValue::from_html_size(font_size as u8)
                )
            ))
        }

        let cellspacing = if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_cellspacing()
        } else {
            None
        };

        if let Some(cellspacing) = cellspacing {
            let width_value = specified::Length::from_px(cellspacing as f32);
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderSpacing(
                    Box::new(border_spacing::SpecifiedValue {
                        horizontal: width_value,
                        vertical: None,
                    }))));
        }


        let size = if let Some(this) = self.downcast::<HTMLInputElement>() {
            // FIXME(pcwalton): More use of atoms, please!
            match (*self.unsafe_get()).get_attr_val_for_layout(&ns!(), &local_name!("type")) {
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
            let value = specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(size));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Width(
                    specified::LengthOrPercentageOrAuto::Length(value))));
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
                    shared_lock,
                    PropertyDeclaration::Width(width_value)));
            }
            LengthOrPercentageOrAuto::Length(length) => {
                let width_value = specified::LengthOrPercentageOrAuto::Length(
                    specified::NoCalcLength::Absolute(length));
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Width(width_value)));
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
                    shared_lock,
                    PropertyDeclaration::Height(height_value)));
            }
            LengthOrPercentageOrAuto::Length(length) => {
                let height_value = specified::LengthOrPercentageOrAuto::Length(
                    specified::NoCalcLength::Absolute(length));
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Height(height_value)));
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
            let value = specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(cols));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Width(specified::LengthOrPercentageOrAuto::Length(value))));
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
            let value = specified::NoCalcLength::FontRelative(specified::FontRelativeLength::Em(rows as CSSFloat));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Height(specified::LengthOrPercentageOrAuto::Length(value))));
        }


        let border = if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_border()
        } else {
            None
        };

        if let Some(border) = border {
            let width_value = specified::BorderWidth::from_length(specified::Length::from_px(border as f32));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderTopWidth(Box::new(width_value.clone()))));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderLeftWidth(Box::new(width_value.clone()))));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderBottomWidth(Box::new(width_value.clone()))));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderRightWidth(Box::new(width_value))));
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

    #[allow(unsafe_code)]
    unsafe fn get_rowspan(self) -> u32 {
        if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_rowspan().unwrap_or(1)
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
    fn style_attribute(&self) -> *const Option<Arc<Locked<PropertyDeclarationBlock>>> {
        unsafe {
            (*self.unsafe_get()).style_attribute.borrow_for_layout()
        }
    }

    #[allow(unsafe_code)]
    fn local_name(&self) -> &LocalName {
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

    #[allow(unsafe_code)]
    fn get_lang_for_layout(&self) -> String {
        unsafe {
            let mut current_node = Some(self.upcast::<Node>());
            while let Some(node) = current_node {
                current_node = node.parent_node_ref();
                match node.downcast::<Element>().map(|el| el.unsafe_get()) {
                    Some(elem) => {
                        if let Some(attr) = (*elem).get_attr_val_for_layout(&ns!(xml), &local_name!("lang")) {
                            return attr.to_owned();
                        }
                        if let Some(attr) = (*elem).get_attr_val_for_layout(&ns!(), &local_name!("lang")) {
                            return attr.to_owned();
                        }
                    }
                    None => continue
                }
            }
            // TODO: Check meta tags for a pragma-set default language
            // TODO: Check HTTP Content-Language header
            String::new()
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
    fn insert_selector_flags(&self, flags: ElementSelectorFlags) {
        debug_assert!(thread_state::get() == thread_state::LAYOUT);
        unsafe {
            let f = &(*self.unsafe_get()).selector_flags;
            f.set(f.get() | flags);
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        unsafe {
            (*self.unsafe_get()).selector_flags.get().contains(flags)
        }
    }
}

impl Element {
    pub fn html_element_in_html_document(&self) -> bool {
        self.namespace == ns!(html) && self.upcast::<Node>().is_in_html_doc()
    }

    pub fn local_name(&self) -> &LocalName {
        &self.local_name
    }

    pub fn parsed_name(&self, mut name: DOMString) -> LocalName {
        if self.html_element_in_html_document() {
            name.make_ascii_lowercase();
        }
        LocalName::from(name)
    }

    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub fn prefix(&self) -> Option<&DOMString> {
        self.prefix.as_ref()
    }

    pub fn attrs(&self) -> Ref<[JS<Attr>]> {
        Ref::map(self.attrs.borrow(), |attrs| &**attrs)
    }

    // Element branch of https://dom.spec.whatwg.org/#locate-a-namespace
    pub fn locate_namespace(&self, prefix: Option<DOMString>) -> Namespace {
        let prefix = prefix.map(String::from).map(LocalName::from);

        let inclusive_ancestor_elements =
            self.upcast::<Node>()
                .inclusive_ancestors()
                .filter_map(Root::downcast::<Self>);

        // Steps 3-4.
        for element in inclusive_ancestor_elements {
            // Step 1.
            if element.namespace() != &ns!() && element.prefix().map(|p| &**p) == prefix.as_ref().map(|p| &**p) {
                return element.namespace().clone();
            }

            // Step 2.
            let attr = ref_filter_map(self.attrs(), |attrs| {
                attrs.iter().find(|attr| {
                    if attr.namespace() != &ns!(xmlns) {
                        return false;
                    }
                    match (attr.prefix(), prefix.as_ref()) {
                        (Some(&namespace_prefix!("xmlns")), Some(prefix)) => {
                            attr.local_name() == prefix
                        },
                        (None, None) => attr.local_name() == &local_name!("xmlns"),
                        _ => false,
                    }
                })
            });

            if let Some(attr) = attr {
                return (**attr.value()).into();
            }
        }

        ns!()
    }

    pub fn style_attribute(&self) -> &DOMRefCell<Option<Arc<Locked<PropertyDeclarationBlock>>>> {
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

            local_name!("area") | local_name!("base") | local_name!("basefont") |
            local_name!("bgsound") | local_name!("br") |
            local_name!("col") |  local_name!("embed") | local_name!("frame") |
            local_name!("hr") | local_name!("img") |
            local_name!("input") | local_name!("keygen") | local_name!("link") |
            local_name!("menuitem") | local_name!("meta") |
            local_name!("param") | local_name!("source") | local_name!("track") |
            local_name!("wbr") => true,
            _ => false
        }
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
                        if attr.prefix() == Some(&namespace_prefix!("xmlns")) &&
                           **attr.value() == *namespace {
                            return Some(attr.LocalName());
                        }
                    }
                },
                None => return None,
            }
        }
        None
    }

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

    pub fn push_new_attribute(&self,
                              local_name: LocalName,
                              value: AttrValue,
                              name: LocalName,
                              namespace: Namespace,
                              prefix: Option<Prefix>) {
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
        self.will_mutate_attr(attr);
        self.attrs.borrow_mut().push(JS::from_ref(attr));
        if attr.namespace() == &ns!() {
            vtable_for(self.upcast()).attribute_mutated(attr, AttributeMutation::Set(None));
        }
    }

    pub fn get_attribute(&self, namespace: &Namespace, local_name: &LocalName) -> Option<Root<Attr>> {
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
                                     prefix: Option<Prefix>) {
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
                LocalName::from(name)
            },
        };
        let value = self.parse_attribute(&qname.ns, &qname.local, value);
        self.push_new_attribute(qname.local, value, name, qname.ns, prefix);
    }

    pub fn set_attribute(&self, name: &LocalName, value: AttrValue) {
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
        let name = LocalName::from(name);
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
                                       local_name: LocalName,
                                       value: AttrValue,
                                       name: LocalName,
                                       namespace: Namespace,
                                       prefix: Option<Prefix>,
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
                           local_name: &LocalName,
                           value: DOMString)
                           -> AttrValue {
        if *namespace == ns!() {
            vtable_for(self.upcast()).parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value.into())
        }
    }

    pub fn remove_attribute(&self, namespace: &Namespace, local_name: &LocalName) -> Option<Root<Attr>> {
        self.remove_first_matching_attribute(|attr| {
            attr.namespace() == namespace && attr.local_name() == local_name
        })
    }

    pub fn remove_attribute_by_name(&self, name: &LocalName) -> Option<Root<Attr>> {
        self.remove_first_matching_attribute(|attr| attr.name() == name)
    }

    fn remove_first_matching_attribute<F>(&self, find: F) -> Option<Root<Attr>>
        where F: Fn(&Attr) -> bool
    {
        let idx = self.attrs.borrow().iter().position(|attr| find(&attr));

        idx.map(|idx| {
            let attr = Root::from_ref(&*(*self.attrs.borrow())[idx]);
            self.will_mutate_attr(&attr);
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
                QuirksMode::NoQuirks | QuirksMode::LimitedQuirks => lhs == rhs,
                QuirksMode::Quirks => lhs.eq_ignore_ascii_case(&rhs),
            }
        };
        self.get_attribute(&ns!(), &local_name!("class"))
            .map_or(false, |attr| attr.value().as_tokens().iter().any(|atom| is_equal(name, atom)))
    }

    pub fn set_atomic_attribute(&self, local_name: &LocalName, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value.into());
        self.set_attribute(local_name, value);
    }

    pub fn has_attribute(&self, local_name: &LocalName) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        self.attrs
            .borrow()
            .iter()
            .any(|attr| attr.local_name() == local_name && attr.namespace() == &ns!())
    }

    pub fn set_bool_attribute(&self, local_name: &LocalName, value: bool) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(local_name, DOMString::new());
        } else {
            self.remove_attribute(&ns!(), local_name);
        }
    }

    pub fn get_url_attribute(&self, local_name: &LocalName) -> DOMString {
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
    pub fn set_url_attribute(&self, local_name: &LocalName, value: DOMString) {
        self.set_string_attribute(local_name, value);
    }

    pub fn get_string_attribute(&self, local_name: &LocalName) -> DOMString {
        match self.get_attribute(&ns!(), local_name) {
            Some(x) => x.Value(),
            None => DOMString::new(),
        }
    }
    pub fn set_string_attribute(&self, local_name: &LocalName, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value.into()));
    }

    pub fn get_tokenlist_attribute(&self, local_name: &LocalName) -> Vec<Atom> {
        self.get_attribute(&ns!(), local_name).map(|attr| {
            attr.value()
                .as_tokens()
                .to_vec()
        }).unwrap_or(vec!())
    }

    pub fn set_tokenlist_attribute(&self, local_name: &LocalName, value: DOMString) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name,
                           AttrValue::from_serialized_tokenlist(value.into()));
    }

    pub fn set_atomic_tokenlist_attribute(&self, local_name: &LocalName, tokens: Vec<Atom>) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens));
    }

    pub fn get_int_attribute(&self, local_name: &LocalName, default: i32) -> i32 {
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

    pub fn set_int_attribute(&self, local_name: &LocalName, value: i32) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::Int(value.to_string(), value));
    }

    pub fn get_uint_attribute(&self, local_name: &LocalName, default: u32) -> u32 {
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
    pub fn set_uint_attribute(&self, local_name: &LocalName, value: u32) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::UInt(value.to_string(), value));
    }

    pub fn will_mutate_attr(&self, attr: &Attr) {
        let node = self.upcast::<Node>();
        node.owner_doc().element_attr_will_change(self, attr);
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
            if doc.quirks_mode() != QuirksMode::Quirks {
                win.scroll(x, y, behavior);
            }

            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == QuirksMode::Quirks &&
           !self.potentially_scrollable() {
               win.scroll(x, y, behavior);
               return;
        }

        // Step 10 (TODO)

        // Step 11
        win.scroll_node(node.to_trusted_node_address(), x, y, behavior);
    }

    // https://w3c.github.io/DOM-Parsing/#parsing
    pub fn parse_fragment(&self, markup: DOMString) -> Fallible<Root<DocumentFragment>> {
        // Steps 1-2.
        let context_document = document_from_node(self);
        // TODO(#11995): XML case.
        let new_children = ServoParser::parse_html_fragment(self, markup);
        // Step 3.
        let fragment = DocumentFragment::new(&context_document);
        // Step 4.
        for child in new_children {
            fragment.upcast::<Node>().AppendChild(&child).unwrap();
        }
        // Step 5.
        Ok(fragment)
    }

    pub fn fragment_parsing_context(owner_doc: &Document, element: Option<&Self>) -> Root<Self> {
        match element {
            Some(elem) if elem.local_name() != &local_name!("html") || !elem.html_element_in_html_document() => {
                Root::from_ref(elem)
            },
            _ => {
                Root::upcast(HTMLBodyElement::new(local_name!("body"), None, owner_doc))
            }
        }
    }

    // https://fullscreen.spec.whatwg.org/#fullscreen-element-ready-check
    pub fn fullscreen_element_ready_check(&self) -> bool {
        if !self.is_connected() {
            return false
        }
        let document = document_from_node(self);
        document.get_allow_fullscreen()
    }

    // https://html.spec.whatwg.org/multipage/#home-subtree
    pub fn is_in_same_home_subtree<T>(&self, other: &T) -> bool
        where T: DerivedFrom<Element> + DomObject
    {
        let other = other.upcast::<Element>();
        self.root_element() == other.root_element()
    }
}

impl ElementMethods for Element {
    // https://dom.spec.whatwg.org/#dom-element-namespaceuri
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        Node::namespace_to_string(self.namespace.clone())
    }

    // https://dom.spec.whatwg.org/#dom-element-localname
    fn LocalName(&self) -> DOMString {
        // FIXME(ajeffrey): Convert directly from LocalName to DOMString
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
                LocalName::from(qualified_name.to_ascii_uppercase())
            } else {
                LocalName::from(qualified_name)
            }
        });
        DOMString::from(&*name)
    }

    // https://dom.spec.whatwg.org/#dom-element-id
    fn Id(&self) -> DOMString {
        self.get_string_attribute(&local_name!("id"))
    }

    // https://dom.spec.whatwg.org/#dom-element-id
    fn SetId(&self, id: DOMString) {
        self.set_atomic_attribute(&local_name!("id"), id);
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(&self) -> DOMString {
        self.get_string_attribute(&local_name!("class"))
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(&self, class: DOMString) {
        self.set_tokenlist_attribute(&local_name!("class"), class);
    }

    // https://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(&self) -> Root<DOMTokenList> {
        self.class_list.or_init(|| DOMTokenList::new(self, &local_name!("class")))
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
        self.get_attribute(namespace, &LocalName::from(local_name))
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
        let qualified_name = LocalName::from(qualified_name);
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
            self.will_mutate_attr(attr);
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
        let local_name = LocalName::from(local_name);
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
        HTMLCollection::by_qualified_name(&window, self.upcast(), LocalName::from(&*localname))
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
        let rect = self.upcast::<Node>().bounding_content_box_or_zero();
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
            if doc.quirks_mode() == QuirksMode::Quirks {
                return 0.0;
            }

            // Step 6
            return win.ScrollY() as f64;
        }

        // Step 7
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == QuirksMode::Quirks &&
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
            if doc.quirks_mode() != QuirksMode::Quirks {
                win.scroll(win.ScrollX() as f64, y, behavior);
            }

            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == QuirksMode::Quirks &&
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
            if doc.quirks_mode() != QuirksMode::Quirks {
                // Step 6
                return win.ScrollX() as f64;
            }

            return 0.0;
        }

        // Step 7
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == QuirksMode::Quirks &&
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
            if doc.quirks_mode() == QuirksMode::Quirks {
                return;
            }

            win.scroll(x, win.ScrollY() as f64, behavior);
            return;
        }

        // Step 9
        if doc.GetBody().r() == self.downcast::<HTMLElement>() &&
           doc.quirks_mode() == QuirksMode::Quirks &&
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
        // Step 1.
        let frag = try!(self.parse_fragment(value));
        // Step 2.
        // https://github.com/w3c/DOM-Parsing/issues/1
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            Root::upcast(template.Content())
        } else {
            Root::from_ref(self.upcast())
        };
        Node::replace_all(Some(frag.upcast()), &target);
        Ok(())
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#widl-Element-outerHTML
    fn GetOuterHTML(&self) -> Fallible<DOMString> {
        self.serialize(IncludeNode)
    }

    // https://w3c.github.io/DOM-Parsing/#dom-element-outerhtml
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
                let body_elem = Element::create(QualName::new(ns!(html), local_name!("body")),
                                                None, &context_document,
                                                ElementCreator::ScriptCreated);
                Root::upcast(body_elem)
            },
            _ => context_node.GetParentElement().unwrap()
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
        match SelectorParser::parse_author_origin_no_namespace(&selectors) {
            Err(()) => Err(Error::Syntax),
            Ok(selectors) => {
                Ok(matches(&selectors.0, &Root::from_ref(self), None))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-webkitmatchesselector
    fn WebkitMatchesSelector(&self, selectors: DOMString) -> Fallible<bool> {
        self.Matches(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        match SelectorParser::parse_author_origin_no_namespace(&selectors) {
            Err(()) => Err(Error::Syntax),
            Ok(selectors) => {
                let root = self.upcast::<Node>();
                for element in root.inclusive_ancestors() {
                    if let Some(element) = Root::downcast::<Element>(element) {
                        if matches(&selectors.0, &element, None)
                        {
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
        let context = Element::fragment_parsing_context(
            &context.owner_doc(), context.downcast::<Element>());

        // Step 3.
        let fragment = try!(context.parse_fragment(text));

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

    // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
    #[allow(unrooted_must_root)]
    fn RequestFullscreen(&self) -> Rc<Promise> {
        let doc = document_from_node(self);
        doc.enter_fullscreen(self)
    }
}

pub fn fragment_affecting_attributes() -> [LocalName; 3] {
    [local_name!("width"), local_name!("height"), local_name!("src")]
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
            &local_name!("style") => {
                // Modifying the `style` attribute might change style.
                *self.style_attribute.borrow_mut() = match mutation {
                    AttributeMutation::Set(..) => {
                        // This is the fast path we use from
                        // CSSStyleDeclaration.
                        //
                        // Juggle a bit to keep the borrow checker happy
                        // while avoiding the extra clone.
                        let is_declaration = match *attr.value() {
                            AttrValue::Declaration(..) => true,
                            _ => false,
                        };

                        let block = if is_declaration {
                            let mut value = AttrValue::String(String::new());
                            attr.swap_value(&mut value);
                            let (serialization, block) = match value {
                                AttrValue::Declaration(s, b) => (s, b),
                                _ => unreachable!(),
                            };
                            let mut value = AttrValue::String(serialization);
                            attr.swap_value(&mut value);
                            block
                        } else {
                            let win = window_from_node(self);
                            Arc::new(doc.style_shared_lock().wrap(parse_style_attribute(
                                &attr.value(),
                                &doc.base_url(),
                                win.css_error_reporter(),
                                ParserContextExtraData::default())))
                        };

                        Some(block)
                    }
                    AttributeMutation::Removed => {
                        None
                    }
                };
            },
            &local_name!("id") => {
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
                   common_style_affecting_attributes().iter().any(|a| &a.attr_name == attr.local_name()) ||
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

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("id") => AttrValue::from_atomic(value.into()),
            &local_name!("class") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if let Some(f) = self.as_maybe_form_control() {
            f.bind_form_control_to_tree();
        }

        if !tree_in_doc {
            return;
        }

        let doc = document_from_node(self);
        if let Some(ref value) = *self.id_attribute.borrow() {
            doc.register_named_element(self, value.clone());
        }
        // This is used for layout optimization.
        doc.increment_dom_count();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if let Some(f) = self.as_maybe_form_control() {
            f.unbind_form_control_from_tree();
        }

        if !context.tree_in_doc {
            return;
        }

        let doc = document_from_node(self);
        let fullscreen = doc.GetFullscreenElement();
        if fullscreen.r() == Some(self) {
            doc.exit_fullscreen();
        }
        if let Some(ref value) = *self.id_attribute.borrow() {
            doc.unregister_named_element(self, value.clone());
        }
        // This is used for layout optimization.
        doc.decrement_dom_count();
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }

        let flags = self.selector_flags.get();
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
    type Impl = SelectorImpl;

    fn match_attr<F>(&self, attr: &AttrSelector<SelectorImpl>, test: F) -> bool
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

    fn get_local_name(&self) -> &LocalName {
        self.local_name()
    }

    fn get_namespace(&self) -> &Namespace {
        self.namespace()
    }

    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    _: &mut StyleRelations,
                                    _: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        match *pseudo_class {
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

            // FIXME(#15746): This is wrong, we need to instead use extended filtering as per RFC4647
            //                https://tools.ietf.org/html/rfc4647#section-3.3.2
            NonTSPseudoClass::Lang(ref lang) => extended_filtering(&*self.get_lang(), &*lang),

            NonTSPseudoClass::ReadOnly =>
                !Element::state(self).contains(pseudo_class.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Fullscreen |
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
        if let Some(ref attr) = self.get_attribute(&ns!(), &local_name!("class")) {
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

    pub fn as_stylesheet_owner(&self) -> Option<&StylesheetOwner> {
        if let Some(s) = self.downcast::<HTMLStyleElement>() {
            return Some(s as &StylesheetOwner)
        }

        if let Some(l) = self.downcast::<HTMLLinkElement>() {
            return Some(l as &StylesheetOwner)
        }

        None
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
                self.has_attribute(&local_name!("href"))
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

    // https://html.spec.whatwg.org/multipage/#language
    pub fn get_lang(&self) -> String {
        self.upcast::<Node>().inclusive_ancestors().filter_map(|node| {
            node.downcast::<Element>().and_then(|el| {
                el.get_attribute(&ns!(xml), &local_name!("lang")).or_else(|| {
                    el.get_attribute(&ns!(), &local_name!("lang"))
                }).map(|attr| String::from(attr.Value()))
            })
        // TODO: Check meta tags for a pragma-set default language
        // TODO: Check HTTP Content-Language header
        }).next().unwrap_or(String::new())
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

    pub fn fullscreen_state(&self) -> bool {
        self.state.get().contains(IN_FULLSCREEN_STATE)
    }

    pub fn set_fullscreen_state(&self, value: bool) {
        self.set_state(IN_FULLSCREEN_STATE, value)
    }

    /// https://dom.spec.whatwg.org/#connected
    pub fn is_connected(&self) -> bool {
        let node = self.upcast::<Node>();
        let root = node.GetRootNode();
        root.is::<Document>()
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
            if let Some(ref legend) = ancestor.children().find(|n| n.is::<HTMLLegendElement>()) {
                // XXXabinader: should we save previous ancestor to avoid this iteration?
                if node.ancestors().any(|ancestor| ancestor == *legend) {
                    continue;
                }
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
        let has_disabled_attrib = self.has_attribute(&local_name!("disabled"));
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }
}

#[derive(Clone, Copy)]
pub enum AttributeMutation<'a> {
    /// The attribute is set, keep track of old value.
    /// https://dom.spec.whatwg.org/#attribute-is-set
    Set(Option<&'a AttrValue>),

    /// The attribute is removed.
    /// https://dom.spec.whatwg.org/#attribute-is-removed
    Removed,
}

impl<'a> AttributeMutation<'a> {
    pub fn is_removal(&self) -> bool {
        match *self {
            AttributeMutation::Removed => true,
            AttributeMutation::Set(..) => false,
        }
    }

    pub fn new_value<'b>(&self, attr: &'b Attr) -> Option<Ref<'b, AttrValue>> {
        match *self {
            AttributeMutation::Set(_) => Some(attr.value()),
            AttributeMutation::Removed => None,
        }
    }
}

/// A holder for an element's "tag name", which will be lazily
/// resolved and cached. Should be reset when the document
/// owner changes.
#[derive(JSTraceable, HeapSizeOf)]
struct TagName {
    ptr: DOMRefCell<Option<LocalName>>,
}

impl TagName {
    fn new() -> TagName {
        TagName { ptr: DOMRefCell::new(None) }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    fn or_init<F>(&self, cb: F) -> LocalName
        where F: FnOnce() -> LocalName
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

pub struct ElementPerformFullscreenEnter {
    element: Trusted<Element>,
    promise: TrustedPromise,
    error: bool,
}

impl ElementPerformFullscreenEnter {
    pub fn new(element: Trusted<Element>, promise: TrustedPromise, error: bool) -> Box<ElementPerformFullscreenEnter> {
        box ElementPerformFullscreenEnter {
            element: element,
            promise: promise,
            error: error,
        }
    }
}

impl Runnable for ElementPerformFullscreenEnter {
    fn name(&self) -> &'static str { "ElementPerformFullscreenEnter" }

    #[allow(unrooted_must_root)]
    fn handler(self: Box<ElementPerformFullscreenEnter>) {
        let element = self.element.root();
        let document = document_from_node(element.r());

        // Step 7.1
        if self.error || !element.fullscreen_element_ready_check() {
            // JSAutoCompartment needs to be manually made.
            // Otherwise, Servo will crash.
            let promise = self.promise.root();
            let promise_cx = promise.global().get_cx();
            let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
            document.upcast::<EventTarget>().fire_event(atom!("fullscreenerror"));
            promise.reject_error(promise.global().get_cx(), Error::Type(String::from("fullscreen is not connected")));
            return
        }

        // TODO Step 7.2-4
        // Step 7.5
        element.set_fullscreen_state(true);
        document.set_fullscreen_element(Some(&element));
        document.window().reflow(ReflowGoal::ForDisplay,
                                 ReflowQueryType::NoQuery,
                                 ReflowReason::ElementStateChanged);

        // Step 7.6
        document.upcast::<EventTarget>().fire_event(atom!("fullscreenchange"));

        // Step 7.7
        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let promise = self.promise.root();
        let promise_cx = promise.global().get_cx();
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        promise.resolve(promise.global().get_cx(), HandleValue::undefined());
    }
}

pub struct ElementPerformFullscreenExit {
    element: Trusted<Element>,
    promise: TrustedPromise,
}

impl ElementPerformFullscreenExit {
    pub fn new(element: Trusted<Element>, promise: TrustedPromise) -> Box<ElementPerformFullscreenExit> {
        box ElementPerformFullscreenExit {
            element: element,
            promise: promise,
        }
    }
}

impl Runnable for ElementPerformFullscreenExit {
    fn name(&self) -> &'static str { "ElementPerformFullscreenExit" }

    #[allow(unrooted_must_root)]
    fn handler(self: Box<ElementPerformFullscreenExit>) {
        let element = self.element.root();
        let document = document_from_node(element.r());
        // TODO Step 9.1-5
        // Step 9.6
        element.set_fullscreen_state(false);

        document.window().reflow(ReflowGoal::ForDisplay,
                                 ReflowQueryType::NoQuery,
                                 ReflowReason::ElementStateChanged);

        document.set_fullscreen_element(None);

        // Step 9.8
        document.upcast::<EventTarget>().fire_event(atom!("fullscreenchange"));

        // Step 9.10
        let promise = self.promise.root();
        // JSAutoCompartment needs to be manually made.
        // Otherwise, Servo will crash.
        let promise_cx = promise.global().get_cx();
        let _ac = JSAutoCompartment::new(promise_cx, promise.reflector().get_jsobject().get());
        promise.resolve(promise.global().get_cx(), HandleValue::undefined());
    }
}

pub fn reflect_cross_origin_attribute(element: &Element) -> Option<DOMString> {
    let attr = element.get_attribute(&ns!(), &local_name!("crossorigin"));

    if let Some(mut val) = attr.map(|v| v.Value()) {
        val.make_ascii_lowercase();
        if val == "anonymous" || val == "use-credentials" {
            return Some(val);
        }
        return Some(DOMString::from("anonymous"));
    }
    None
}

pub fn set_cross_origin_attribute(element: &Element, value: Option<DOMString>) {
    match value {
        Some(val) => element.set_string_attribute(&local_name!("crossorigin"), val),
        None => {
            element.remove_attribute(&ns!(), &local_name!("crossorigin"));
        }
    }
}

pub fn cors_setting_for_element(element: &Element) -> Option<CorsSettings> {
    reflect_cross_origin_attribute(element).map_or(None, |attr| {
        match &*attr {
            "anonymous" => Some(CorsSettings::Anonymous),
            "use-credentials" => Some(CorsSettings::UseCredentials),
            _ => unreachable!()
        }
    })
}
