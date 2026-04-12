/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Element nodes.

use std::borrow::Cow;
use std::cell::{Cell, LazyCell};
use std::default::Default;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, mem};

use app_units::Au;
use cssparser::match_ignore_ascii_case;
use devtools_traits::{AttrInfo, DomMutation, ScriptToDevtoolsControlMsg};
use dom_struct::dom_struct;
use euclid::Rect;
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::{LocalName, Namespace, Prefix, QualName, local_name, namespace_prefix, ns};
use js::context::JSContext;
use js::jsapi::{Heap, JSAutoRealm};
use js::jsval::JSVal;
use js::rust::HandleObject;
use layout_api::{LayoutDamage, QueryMsg, ScrollContainerQueryFlags, StyleData, with_layout_state};
use net_traits::ReferrerPolicy;
use net_traits::request::{CorsSettings, CredentialsMode};
use selectors::attr::CaseSensitivity;
use selectors::matching::ElementSelectorFlags;
use selectors::sink::Push;
use servo_arc::Arc as ServoArc;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::properties::longhands::{
    self, background_image, border_spacing, color, font_family, font_size,
};
use style::properties::{
    ComputedValues, Importance, PropertyDeclaration, PropertyDeclarationBlock,
    parse_style_attribute,
};
use style::rule_tree::{CascadeLevel, CascadeOrigin};
use style::selector_parser::{RestyleDamage, SelectorParser, Snapshot};
use style::shared_lock::Locked;
use style::stylesheets::layer_rule::LayerOrder;
use style::stylesheets::{CssRuleType, UrlExtraData};
use style::values::computed::Overflow;
use style::values::generics::NonNegative;
use style::values::generics::position::PreferredRatio;
use style::values::generics::ratio::Ratio;
use style::values::{AtomIdent, CSSFloat, computed, specified};
use style::{ArcSlice, CaseSensitivityExt, dom_apis, thread_state};
use style_traits::CSSPixel;
use stylo_atoms::Atom;
use stylo_dom::ElementState;
use xml5ever::serialize::TraversalScope::{
    ChildrenOnly as XmlChildrenOnly, IncludeNode as XmlIncludeNode,
};

use crate::conversions::Convert;
use crate::dom::activation::Activatable;
use crate::dom::attr::{Attr, is_relevant_attribute};
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::{
    ElementMethods, GetHTMLOptions, ScrollIntoViewContainer, ScrollLogicalPosition, ShadowRootInit,
};
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMethods, ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    ScrollBehavior, ScrollToOptions, WindowMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    BooleanOrScrollIntoViewOptions, NodeOrString, TrustedHTMLOrNullIsEmptyString,
    TrustedHTMLOrString,
    TrustedHTMLOrTrustedScriptOrTrustedScriptURLOrString as TrustedTypeOrString,
};
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::domname::{
    self, is_valid_attribute_local_name, namespace_from_domstring,
};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom, ToLayout};
use crate::dom::bindings::str::DOMString;
use crate::dom::create::create_element;
use crate::dom::csp::{CspReporting, InlineCheckType, SourcePosition};
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReaction, CustomElementRegistry,
    CustomElementState, is_valid_custom_element_name,
};
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domrect::DOMRect;
use crate::dom::domrectlist::DOMRectList;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::elementinternals::ElementInternals;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlanchorelement::HTMLAnchorElement;
use crate::dom::html::htmlbodyelement::HTMLBodyElement;
use crate::dom::html::htmlbuttonelement::HTMLButtonElement;
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::html::htmlfontelement::HTMLFontElement;
use crate::dom::html::htmlformelement::FormControlElementHelpers;
use crate::dom::html::htmlhrelement::{HTMLHRElement, SizePresentationalHint};
use crate::dom::html::htmliframeelement::HTMLIFrameElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmllabelelement::HTMLLabelElement;
use crate::dom::html::htmllegendelement::HTMLLegendElement;
use crate::dom::html::htmllinkelement::HTMLLinkElement;
use crate::dom::html::htmlobjectelement::HTMLObjectElement;
use crate::dom::html::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::html::htmloutputelement::HTMLOutputElement;
use crate::dom::html::htmlscriptelement::HTMLScriptElement;
use crate::dom::html::htmlselectelement::HTMLSelectElement;
use crate::dom::html::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::html::htmlstyleelement::HTMLStyleElement;
use crate::dom::html::htmltablecellelement::HTMLTableCellElement;
use crate::dom::html::htmltablecolelement::HTMLTableColElement;
use crate::dom::html::htmltableelement::HTMLTableElement;
use crate::dom::html::htmltablerowelement::HTMLTableRowElement;
use crate::dom::html::htmltablesectionelement::HTMLTableSectionElement;
use crate::dom::html::htmltemplateelement::HTMLTemplateElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::input_element::HTMLInputElement;
use crate::dom::intersectionobserver::{IntersectionObserver, IntersectionObserverRegistration};
use crate::dom::mutationobserver::{Mutation, MutationObserver};
use crate::dom::namednodemap::NamedNodeMap;
use crate::dom::node::{
    BindContext, ChildrenMutation, CloneChildrenFlag, IsShadowTree, Node, NodeDamage, NodeFlags,
    NodeTraits, ShadowIncluding, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::promise::Promise;
use crate::dom::range::Range;
use crate::dom::raredata::ElementRareData;
use crate::dom::scrolling_box::{ScrollAxisState, ScrollingBox};
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::{IsUserAgentWidget, ShadowRoot};
use crate::dom::svg::svgsvgelement::SVGSVGElement;
use crate::dom::text::Text;
use crate::dom::trustedtypes::trustedhtml::TrustedHTML;
use crate::dom::trustedtypes::trustedtypepolicyfactory::TrustedTypePolicyFactory;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::{VirtualMethods, vtable_for};
use crate::layout_dom::ServoDangerousStyleElement;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;
use crate::stylesheet_loader::StylesheetOwner;

// TODO: Update focus state when the top-level browsing context gains or loses system focus,
// and when the element enters or leaves a browsing context container.
// https://html.spec.whatwg.org/multipage/#selector-focus

/// <https://dom.spec.whatwg.org/#element>
#[dom_struct]
pub struct Element {
    node: Node,
    #[no_trace]
    local_name: LocalName,
    tag_name: TagName,
    #[no_trace]
    namespace: Namespace,
    #[no_trace]
    prefix: DomRefCell<Option<Prefix>>,
    attrs: DomRefCell<Vec<Dom<Attr>>>,
    #[no_trace]
    id_attribute: DomRefCell<Option<Atom>>,
    /// <https://dom.spec.whatwg.org/#concept-element-is-value>
    #[no_trace]
    is: DomRefCell<Option<LocalName>>,
    #[conditional_malloc_size_of]
    #[no_trace]
    style_attribute: DomRefCell<Option<ServoArc<Locked<PropertyDeclarationBlock>>>>,
    attr_list: MutNullableDom<NamedNodeMap>,
    class_list: MutNullableDom<DOMTokenList>,
    #[no_trace]
    state: Cell<ElementState>,
    /// These flags are set by the style system to indicate the that certain
    /// operations may require restyling this element or its descendants.
    selector_flags: AtomicUsize,
    rare_data: DomRefCell<Option<Box<ElementRareData>>>,

    /// Style data for this node. This is accessed and mutated by style
    /// passes and is used to lay out this node and populate layout data.
    #[no_trace]
    style_data: DomRefCell<Option<Box<StyleData>>>,
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.local_name)?;
        if let Some(ref id) = *self.id_attribute.borrow() {
            write!(f, " id={}", id)?;
        }
        write!(f, ">")
    }
}

#[derive(MallocSizeOf, PartialEq)]
pub(crate) enum ElementCreator {
    ParserCreated(u64),
    ScriptCreated,
}

pub(crate) enum CustomElementCreationMode {
    Synchronous,
    Asynchronous,
}

impl ElementCreator {
    pub(crate) fn is_parser_created(&self) -> bool {
        match *self {
            ElementCreator::ParserCreated(_) => true,
            ElementCreator::ScriptCreated => false,
        }
    }
    pub(crate) fn return_line_number(&self) -> u64 {
        match *self {
            ElementCreator::ParserCreated(l) => l,
            ElementCreator::ScriptCreated => 1,
        }
    }
}

pub(crate) enum AdjacentPosition {
    BeforeBegin,
    AfterEnd,
    AfterBegin,
    BeforeEnd,
}

impl FromStr for AdjacentPosition {
    type Err = Error;

    fn from_str(position: &str) -> Result<Self, Self::Err> {
        match_ignore_ascii_case! { position,
            "beforebegin" => Ok(AdjacentPosition::BeforeBegin),
            "afterbegin"  => Ok(AdjacentPosition::AfterBegin),
            "beforeend"   => Ok(AdjacentPosition::BeforeEnd),
            "afterend"    => Ok(AdjacentPosition::AfterEnd),
            _             => Err(Error::Syntax(None))
        }
    }
}

//
// Element methods
//
impl Element {
    pub(crate) fn create(
        cx: &mut JSContext,
        name: QualName,
        is: Option<LocalName>,
        document: &Document,
        creator: ElementCreator,
        mode: CustomElementCreationMode,
        proto: Option<HandleObject>,
    ) -> DomRoot<Element> {
        create_element(cx, name, is, document, creator, mode, proto)
    }

    pub(crate) fn new_inherited(
        local_name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> Element {
        Element::new_inherited_with_state(
            ElementState::empty(),
            local_name,
            namespace,
            prefix,
            document,
        )
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        local_name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> Element {
        Element {
            node: Node::new_inherited(document),
            local_name,
            tag_name: TagName::new(),
            namespace,
            prefix: DomRefCell::new(prefix),
            attrs: DomRefCell::new(vec![]),
            id_attribute: DomRefCell::new(None),
            is: DomRefCell::new(None),
            style_attribute: DomRefCell::new(None),
            attr_list: Default::default(),
            class_list: Default::default(),
            state: Cell::new(state),
            selector_flags: Default::default(),
            rare_data: Default::default(),
            style_data: Default::default(),
        }
    }

    pub(crate) fn set_had_duplicate_attributes(&self) {
        self.ensure_rare_data().had_duplicate_attributes = true;
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<Element> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(Element::new_inherited(
                local_name, namespace, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn rare_data(&self) -> Ref<'_, Option<Box<ElementRareData>>> {
        self.rare_data.borrow()
    }

    fn rare_data_mut(&self) -> RefMut<'_, Option<Box<ElementRareData>>> {
        self.rare_data.borrow_mut()
    }

    fn ensure_rare_data(&self) -> RefMut<'_, Box<ElementRareData>> {
        let mut rare_data = self.rare_data.borrow_mut();
        if rare_data.is_none() {
            *rare_data = Some(Default::default());
        }
        RefMut::map(rare_data, |rare_data| rare_data.as_mut().unwrap())
    }

    pub(crate) fn clean_up_style_data(&self) {
        self.style_data.borrow_mut().take();
    }

    pub(crate) fn restyle(&self, damage: NodeDamage) {
        let doc = self.node.owner_doc();
        let mut restyle = doc.ensure_pending_restyle(self);

        // FIXME(bholley): I think we should probably only do this for
        // NodeStyleDamaged, but I'm preserving existing behavior.
        restyle.hint.insert(RestyleHint::RESTYLE_SELF);

        match damage {
            NodeDamage::Style => {},
            NodeDamage::ContentOrHeritage => {
                doc.note_node_with_dirty_descendants(self.upcast());
                restyle
                    .damage
                    .insert(LayoutDamage::descendant_has_box_damage());
            },
            NodeDamage::Other => {
                doc.note_node_with_dirty_descendants(self.upcast());
                restyle.damage.insert(RestyleDamage::reconstruct());
            },
        }
    }

    pub(crate) fn set_is(&self, is: LocalName) {
        *self.is.borrow_mut() = Some(is);
    }

    /// <https://dom.spec.whatwg.org/#concept-element-is-value>
    pub(crate) fn get_is(&self) -> Option<LocalName> {
        self.is.borrow().clone()
    }

    /// This is a performance optimization. `Element::create` can simply call
    /// `element.set_custom_element_state(CustomElementState::Uncustomized)` to initialize
    /// uncustomized, built-in elements with the right state, which currently just means that the
    /// `DEFINED` state should be `true` for styling. However `set_custom_element_state` has a high
    /// performance cost and it is unnecessary if the element is being created as an uncustomized
    /// built-in element.
    ///
    /// See <https://github.com/servo/servo/issues/37745> for more details.
    pub(crate) fn set_initial_custom_element_state_to_uncustomized(&self) {
        let mut state = self.state.get();
        state.insert(ElementState::DEFINED);
        self.state.set(state);
    }

    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
    pub(crate) fn set_custom_element_state(&self, state: CustomElementState) {
        // no need to inflate rare data for uncustomized
        if state != CustomElementState::Uncustomized {
            self.ensure_rare_data().custom_element_state = state;
        }

        let in_defined_state = matches!(
            state,
            CustomElementState::Uncustomized | CustomElementState::Custom
        );
        self.set_state(ElementState::DEFINED, in_defined_state)
    }

    pub(crate) fn get_custom_element_state(&self) -> CustomElementState {
        if let Some(rare_data) = self.rare_data().as_ref() {
            return rare_data.custom_element_state;
        }
        CustomElementState::Uncustomized
    }

    /// <https://dom.spec.whatwg.org/#concept-element-custom>
    pub(crate) fn is_custom(&self) -> bool {
        self.get_custom_element_state() == CustomElementState::Custom
    }

    pub(crate) fn set_custom_element_definition(&self, definition: Rc<CustomElementDefinition>) {
        self.ensure_rare_data().custom_element_definition = Some(definition);
    }

    pub(crate) fn get_custom_element_definition(&self) -> Option<Rc<CustomElementDefinition>> {
        self.rare_data().as_ref()?.custom_element_definition.clone()
    }

    pub(crate) fn clear_custom_element_definition(&self) {
        self.ensure_rare_data().custom_element_definition = None;
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn push_callback_reaction(&self, function: Rc<Function>, args: Box<[Heap<JSVal>]>) {
        self.ensure_rare_data()
            .custom_element_reaction_queue
            .push(CustomElementReaction::Callback(function, args));
    }

    pub(crate) fn push_upgrade_reaction(&self, definition: Rc<CustomElementDefinition>) {
        self.ensure_rare_data()
            .custom_element_reaction_queue
            .push(CustomElementReaction::Upgrade(definition));
    }

    pub(crate) fn clear_reaction_queue(&self) {
        if let Some(ref mut rare_data) = *self.rare_data_mut() {
            rare_data.custom_element_reaction_queue.clear();
        }
    }

    pub(crate) fn invoke_reactions(&self, cx: &mut JSContext) {
        loop {
            rooted_vec!(let mut reactions);
            match *self.rare_data_mut() {
                Some(ref mut data) => {
                    mem::swap(&mut *reactions, &mut data.custom_element_reaction_queue)
                },
                None => break,
            };

            if reactions.is_empty() {
                break;
            }

            for reaction in reactions.iter() {
                reaction.invoke(cx, self);
            }

            reactions.clear();
        }
    }

    // https://drafts.csswg.org/cssom-view/#css-layout-box
    pub(crate) fn has_css_layout_box(&self) -> bool {
        self.style()
            .is_some_and(|s| !s.get_box().clone_display().is_none())
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    pub(crate) fn is_potentially_scrollable_body(&self) -> bool {
        self.is_potentially_scrollable_body_shared_logic(false)
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    pub(crate) fn is_potentially_scrollable_body_for_scrolling_element(&self) -> bool {
        self.is_potentially_scrollable_body_shared_logic(true)
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    fn is_potentially_scrollable_body_shared_logic(
        &self,
        treat_overflow_clip_on_parent_as_hidden: bool,
    ) -> bool {
        let node = self.upcast::<Node>();
        debug_assert!(
            node.owner_doc().GetBody().as_deref() == self.downcast::<HTMLElement>(),
            "Called is_potentially_scrollable_body on element that is not the <body>"
        );

        // "An element body (which will be the body element) is potentially
        // scrollable if all of the following conditions are true:
        //  - body has an associated box."
        if !self.has_css_layout_box() {
            return false;
        }

        // " - body’s parent element’s computed value of the overflow-x or
        //     overflow-y properties is neither visible nor clip."
        if let Some(parent) = node.GetParentElement() {
            if let Some(style) = parent.style() {
                let mut overflow_x = style.get_box().clone_overflow_x();
                let mut overflow_y = style.get_box().clone_overflow_y();

                // This fulfills the 'treat parent element overflow:clip as overflow:hidden' stipulation
                // from the document.scrollingElement specification.
                if treat_overflow_clip_on_parent_as_hidden {
                    if overflow_x == Overflow::Clip {
                        overflow_x = Overflow::Hidden;
                    }
                    if overflow_y == Overflow::Clip {
                        overflow_y = Overflow::Hidden;
                    }
                }

                if !overflow_x.is_scrollable() && !overflow_y.is_scrollable() {
                    return false;
                }
            };
        }

        // " - body’s computed value of the overflow-x or overflow-y properties
        //     is neither visible nor clip."
        if let Some(style) = self.style() {
            if !style.get_box().clone_overflow_x().is_scrollable() &&
                !style.get_box().clone_overflow_y().is_scrollable()
            {
                return false;
            }
        };

        true
    }

    /// Whether this element is styled such that it establishes a scroll container.
    /// <https://www.w3.org/TR/css-overflow-3/#scroll-container>
    pub(crate) fn establishes_scroll_container(&self) -> bool {
        // The CSS computed value has made sure that either both axes are scrollable or none are scrollable.
        self.upcast::<Node>()
            .effective_overflow()
            .is_some_and(|overflow| overflow.establishes_scroll_container())
    }

    pub(crate) fn has_overflow(&self) -> bool {
        self.ScrollHeight() > self.ClientHeight() || self.ScrollWidth() > self.ClientWidth()
    }

    /// Whether or not this element has a scrolling box according to
    /// <https://drafts.csswg.org/cssom-view/#scrolling-box>.
    ///
    /// This is true if:
    ///  1. The element has a layout box.
    ///  2. The style specifies that overflow should be scrollable (`auto`, `hidden` or `scroll`).
    ///  3. The fragment actually has content that overflows the box.
    fn has_scrolling_box(&self) -> bool {
        self.has_css_layout_box() && self.establishes_scroll_container() && self.has_overflow()
    }

    pub(crate) fn shadow_root(&self) -> Option<DomRoot<ShadowRoot>> {
        self.rare_data()
            .as_ref()?
            .shadow_root
            .as_ref()
            .map(|sr| DomRoot::from_ref(&**sr))
    }

    pub(crate) fn is_shadow_host(&self) -> bool {
        self.shadow_root().is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-element-attachshadow>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn attach_shadow(
        &self,
        cx: &mut JSContext,
        is_ua_widget: IsUserAgentWidget,
        mode: ShadowRootMode,
        clonable: bool,
        serializable: bool,
        delegates_focus: bool,
        slot_assignment_mode: SlotAssignmentMode,
    ) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1. If element’s namespace is not the HTML namespace,
        // then throw a "NotSupportedError" DOMException.
        if self.namespace != ns!(html) {
            return Err(Error::NotSupported(Some(
                "Cannot attach shadow roots to elements with non-HTML namespaces".to_owned(),
            )));
        }

        // Step 2. If element’s local name is not a valid shadow host name,
        // then throw a "NotSupportedError" DOMException.
        if !is_valid_shadow_host_name(self.local_name()) {
            // UA shadow roots may be attached to anything
            if is_ua_widget != IsUserAgentWidget::Yes {
                let error_message = format!(
                    "Cannot attach shadow roots to <{}> elements",
                    *self.local_name()
                );
                return Err(Error::NotSupported(Some(error_message)));
            }
        }

        // Step 3. If element’s local name is a valid custom element name,
        // or element’s is value is non-null
        if is_valid_custom_element_name(self.local_name()) || self.get_is().is_some() {
            // Step 3.1. Let definition be the result of looking up a custom element definition
            // given element’s node document, its namespace, its local name, and its is value.

            let definition = self.get_custom_element_definition();
            // Step 3.2. If definition is not null and definition’s disable shadow
            //  is true, then throw a "NotSupportedError" DOMException.
            if definition.is_some_and(|definition| definition.disable_shadow) {
                let error_message = format!(
                    "The custom element constructor of <{}> disabled attachment of shadow roots",
                    self.local_name()
                );
                return Err(Error::NotSupported(Some(error_message)));
            }
        }

        // Step 4. If element is a shadow host:
        // Step 4.1. Let currentShadowRoot be element’s shadow root.
        if let Some(current_shadow_root) = self.shadow_root() {
            // Step 4.2. If currentShadowRoot’s declarative is false
            // or currentShadowRoot’s mode is not mode
            // then throw a "NotSupportedError" DOMException.
            if !current_shadow_root.is_declarative() ||
                current_shadow_root.shadow_root_mode() != mode
            {
                return Err(Error::NotSupported(Some(
                    "Cannot attach a second shadow root to the same element".into(),
                )));
            }

            // Step 4.3.1. Remove all of currentShadowRoot’s children, in tree order.
            for child in current_shadow_root.upcast::<Node>().children() {
                child.remove_self(cx);
            }

            // Step 4.3.2. Set currentShadowRoot’s declarative to false.
            current_shadow_root.set_declarative(false);

            // Step 4.3.3. Return
            return Ok(current_shadow_root);
        }

        // Step 5. Let shadow be a new shadow root whose node document
        // is element’s node document, host is element, and mode is mode
        //
        // Step 8. Set shadow’s slot assignment to slotAssignment
        //
        // Step 10. Set shadow’s clonable to clonable
        let shadow_root = ShadowRoot::new(
            self,
            &self.node.owner_doc(),
            mode,
            slot_assignment_mode,
            clonable,
            is_ua_widget,
            CanGc::from_cx(cx),
        );

        // This is not in the specification, but this is where we ensure that the
        // non-shadow-tree children of `self` no longer have layout boxes as they are no
        // longer in the flat tree.
        let node = self.upcast::<Node>();
        if node.is_connected() {
            node.remove_layout_boxes_from_subtree(cx.no_gc());
        }
        // Step 6. Set shadow's delegates focus to delegatesFocus
        shadow_root.set_delegates_focus(delegates_focus);

        // Step 7. If element’s custom element state is "precustomized" or "custom",
        // then set shadow’s available to element internals to true.
        if matches!(
            self.get_custom_element_state(),
            CustomElementState::Precustomized | CustomElementState::Custom
        ) {
            shadow_root.set_available_to_element_internals(true);
        }

        // Step 9. Set shadow's declarative to false
        shadow_root.set_declarative(false);

        // Step 11. Set shadow's serializable to serializable
        shadow_root.set_serializable(serializable);

        // Step 12. Set element’s shadow root to shadow
        self.ensure_rare_data().shadow_root = Some(Dom::from_ref(&*shadow_root));
        shadow_root
            .upcast::<Node>()
            .set_containing_shadow_root(Some(&shadow_root));

        let bind_context = BindContext::new(self.upcast(), IsShadowTree::Yes);
        shadow_root.bind_to_tree(cx, &bind_context);

        node.dirty(NodeDamage::Other);

        Ok(shadow_root)
    }

    /// Attach a UA widget shadow root with its default parameters.
    /// Additionally mark ShadowRoot to use styling configuration for a UA widget.
    ///
    /// The general trait of these elements is that it would hide the implementation.
    /// Thus, we would make it inaccessible (i.e., closed mode, not cloneable, and
    /// not serializable).
    ///
    /// With UA shadow root element being assumed as one element, any focus should
    /// be delegated to its host.
    ///
    // TODO: Ideally, all of the UA shadow root should use UA widget styling, but
    //       some of the UA widget implemented prior to the implementation of Gecko's
    //       UA widget matching might need some tweaking.
    // FIXME: We are yet to implement more complex focusing with that is necessary
    //        for delegate focus, and we are using workarounds for that right now.
    pub(crate) fn attach_ua_shadow_root(
        &self,
        cx: &mut JSContext,
        use_ua_widget_styling: bool,
    ) -> DomRoot<ShadowRoot> {
        let root = self
            .attach_shadow(
                cx,
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
            )
            .expect("Attaching UA shadow root failed");

        root.upcast::<Node>()
            .set_in_ua_widget(use_ua_widget_styling);
        root
    }

    // https://html.spec.whatwg.org/multipage/#translation-mode
    pub(crate) fn is_translate_enabled(&self) -> bool {
        let name = &local_name!("translate");
        if self.has_attribute(name) {
            let attribute = self.get_string_attribute(name);
            match_ignore_ascii_case! { &*attribute.str(),
                "yes" | "" => return true,
                "no" => return false,
                _ => {},
            }
        }
        if let Some(parent) = self.upcast::<Node>().GetParentNode() {
            if let Some(elem) = parent.downcast::<Element>() {
                return elem.is_translate_enabled();
            }
        }
        true
    }

    // https://html.spec.whatwg.org/multipage/#the-directionality
    pub(crate) fn directionality(&self) -> String {
        self.downcast::<HTMLElement>()
            .and_then(|html_element| html_element.directionality())
            .unwrap_or_else(|| {
                let node = self.upcast::<Node>();
                node.parent_directionality()
            })
    }

    pub(crate) fn is_root(&self) -> bool {
        match self.node.GetParentNode() {
            None => false,
            Some(node) => node.is::<Document>(),
        }
    }

    /// Return all IntersectionObserverRegistration for this element.
    /// Lazily initialize the raredata if it does not exist.
    pub(crate) fn registered_intersection_observers_mut(
        &self,
    ) -> RefMut<'_, Vec<IntersectionObserverRegistration>> {
        RefMut::map(self.ensure_rare_data(), |rare_data| {
            &mut rare_data.registered_intersection_observers
        })
    }

    pub(crate) fn registered_intersection_observers(
        &self,
    ) -> Option<Ref<'_, Vec<IntersectionObserverRegistration>>> {
        let rare_data: Ref<'_, _> = self.rare_data.borrow();

        if rare_data.is_none() {
            return None;
        }
        Some(Ref::map(rare_data, |rare_data| {
            &rare_data
                .as_ref()
                .unwrap()
                .registered_intersection_observers
        }))
    }

    pub(crate) fn get_intersection_observer_registration(
        &self,
        observer: &IntersectionObserver,
    ) -> Option<Ref<'_, IntersectionObserverRegistration>> {
        if let Some(registrations) = self.registered_intersection_observers() {
            registrations
                .iter()
                .position(|reg_obs| reg_obs.observer == observer)
                .map(|index| Ref::map(registrations, |registrations| &registrations[index]))
        } else {
            None
        }
    }

    /// Add a new IntersectionObserverRegistration with initial value to the element.
    pub(crate) fn add_initial_intersection_observer_registration(
        &self,
        observer: &IntersectionObserver,
    ) {
        self.ensure_rare_data()
            .registered_intersection_observers
            .push(IntersectionObserverRegistration::new_initial(observer));
    }

    /// Removes a certain IntersectionObserver.
    pub(crate) fn remove_intersection_observer(&self, observer: &IntersectionObserver) {
        self.ensure_rare_data()
            .registered_intersection_observers
            .retain(|reg_obs| *reg_obs.observer != *observer)
    }

    /// Get the [`ScrollingBox`] that contains this element, if one does. `position:
    /// fixed` elements do not have a containing [`ScrollingBox`].
    pub(crate) fn scrolling_box(&self, flags: ScrollContainerQueryFlags) -> Option<ScrollingBox> {
        self.owner_window()
            .scrolling_box_query(Some(self.upcast()), flags)
    }

    /// <https://drafts.csswg.org/cssom-view/#scroll-a-target-into-view>
    pub(crate) fn scroll_into_view_with_options(
        &self,
        behavior: ScrollBehavior,
        block: ScrollAxisState,
        inline: ScrollAxisState,
        container: Option<&Element>,
        inner_target_rect: Option<Rect<Au, CSSPixel>>,
    ) {
        let get_target_rect = || match inner_target_rect {
            None => self.upcast::<Node>().border_box().unwrap_or_default(),
            Some(inner_target_rect) => inner_target_rect.translate(
                self.upcast::<Node>()
                    .content_box()
                    .unwrap_or_default()
                    .origin
                    .to_vector(),
            ),
        };

        // Step 1: For each ancestor element or viewport that establishes a scrolling box `scrolling
        // box`, in order of innermost to outermost scrolling box, run these substeps:
        let mut parent_scrolling_box = self.scrolling_box(ScrollContainerQueryFlags::empty());
        while let Some(scrolling_box) = parent_scrolling_box {
            parent_scrolling_box = scrolling_box.parent();

            // Step 1.1: If the Document associated with `target` is not same origin with the
            // Document associated with the element or viewport associated with `scrolling box`,
            // terminate these steps.
            //
            // TODO: Handle this. We currently do not chain up to parent Documents.

            // Step 1.2 Let `position` be the scroll position resulting from running the steps to
            // determine the scroll-into-view position of `target` with `behavior` as the scroll
            // behavior, `block` as the block flow position, `inline` as the inline base direction
            // position and `scrolling box` as the scrolling box.
            let position =
                scrolling_box.determine_scroll_into_view_position(block, inline, get_target_rect());

            // Step 1.3: If `position` is not the same as `scrolling box`’s current scroll position, or
            // `scrolling box` has an ongoing smooth scroll,
            //
            // TODO: Handle smooth scrolling.
            if position != scrolling_box.scroll_position() {
                //  ↪ If `scrolling box` is associated with an element
                //    Perform a scroll of the element’s scrolling box to `position`,
                //    with the `element` as the associated element and `behavior` as the
                //    scroll behavior.
                //  ↪ If `scrolling box` is associated with a viewport
                //    Step 1: Let `document` be the viewport’s associated Document.
                //    Step 2: Let `root element` be document’s root element, if there is one, or
                //    null otherwise.
                //    Step 3: Perform a scroll of the viewport to `position`, with `root element`
                //    as the associated element and `behavior` as the scroll behavior.
                scrolling_box.scroll_to(position, behavior);
            }

            // Step 1.4: If `container` is not null and either `scrolling box` is a shadow-including
            // inclusive ancestor of `container` or is a viewport whose document is a shadow-including
            // inclusive ancestor of `container`, abort the rest of these steps.
            if container.is_some_and(|container| {
                let container_node = container.upcast::<Node>();
                scrolling_box
                    .node()
                    .is_shadow_including_inclusive_ancestor_of(container_node)
            }) {
                return;
            }
        }

        let window_proxy = self.owner_window().window_proxy();
        let Some(frame_element) = window_proxy.frame_element() else {
            return;
        };

        let inner_target_rect = Some(get_target_rect());
        let parent_window = frame_element.owner_window();
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *parent_window.reflector().get_jsobject());
        frame_element.scroll_into_view_with_options(
            behavior,
            block,
            inline,
            None,
            inner_target_rect,
        )
    }

    pub(crate) fn ensure_contenteditable_selection_range(
        &self,
        document: &Document,
        can_gc: CanGc,
    ) -> DomRoot<Range> {
        self.ensure_rare_data()
            .contenteditable_selection_range
            .or_init(|| Range::new_with_doc(document, None, can_gc))
    }

    /// <https://drafts.csswg.org/cssom-view/#scrolling-events>
    ///
    /// > Whenever an element gets scrolled (whether in response to user interaction or
    /// > by an API), the user agent must run these steps:
    pub(crate) fn handle_scroll_event(&self) {
        // Step 1: Let doc be the element’s node document.
        let document = self.owner_document();

        // Step 2: If the element is a snap container, run the steps to update
        // scrollsnapchanging targets for the element with the element’s eventual
        // snap target in the block axis as newBlockTarget and the element’s eventual
        // snap target in the inline axis as newInlineTarget.
        //
        // TODO(#7673): Implement scroll snapping

        // Steps 3 and 4 are shared with other scroll targets.
        document.finish_handle_scroll_event(self.upcast());
    }

    pub(crate) fn style(&self) -> Option<ServoArc<ComputedValues>> {
        self.owner_window().layout_reflow(QueryMsg::StyleQuery);
        self.style_data
            .borrow()
            .as_ref()
            .map(|data| data.element_data.borrow().styles.primary().clone())
    }

    pub(crate) fn is_styled(&self) -> bool {
        self.style_data.borrow().is_some()
    }

    pub(crate) fn is_display_none(&self) -> bool {
        self.style_data.borrow().as_ref().is_none_or(|data| {
            data.element_data
                .borrow()
                .styles
                .primary()
                .get_box()
                .display
                .is_none()
        })
    }
}

/// <https://dom.spec.whatwg.org/#valid-shadow-host-name>
#[inline]
pub(crate) fn is_valid_shadow_host_name(name: &LocalName) -> bool {
    // > A valid shadow host name is:
    // > - a valid custom element name
    if is_valid_custom_element_name(name) {
        return true;
    }

    // > - "article", "aside", "blockquote", "body", "div", "footer", "h1", "h2", "h3",
    // >   "h4", "h5", "h6", "header", "main", "nav", "p", "section", or "span"
    matches!(
        name,
        &local_name!("article") |
            &local_name!("aside") |
            &local_name!("blockquote") |
            &local_name!("body") |
            &local_name!("div") |
            &local_name!("footer") |
            &local_name!("h1") |
            &local_name!("h2") |
            &local_name!("h3") |
            &local_name!("h4") |
            &local_name!("h5") |
            &local_name!("h6") |
            &local_name!("header") |
            &local_name!("main") |
            &local_name!("nav") |
            &local_name!("p") |
            &local_name!("section") |
            &local_name!("span")
    )
}

#[inline]
pub(crate) fn get_attr_for_layout<'dom>(
    elem: LayoutDom<'dom, Element>,
    namespace: &Namespace,
    name: &LocalName,
) -> Option<&'dom AttrValue> {
    elem.attrs()
        .iter()
        .find(|attr| name == attr.local_name() && namespace == attr.namespace())
        .map(|attr| attr.value())
}

impl<'dom> LayoutDom<'dom, Element> {
    #[inline]
    pub(crate) fn is_root(&self) -> bool {
        self.upcast::<Node>()
            .parent_node_ref()
            .is_some_and(|parent| matches!(parent.type_id_for_layout(), NodeTypeId::Document(_)))
    }

    /// Returns true if this element is the body child of an html element root element.
    pub(crate) fn is_body_element_of_html_element_root(&self) -> bool {
        if self.local_name() != &local_name!("body") {
            return false;
        }
        let Some(parent_node) = self.upcast::<Node>().parent_node_ref() else {
            return false;
        };
        let Some(parent_element) = parent_node.downcast::<Element>() else {
            return false;
        };
        parent_element.local_name() == &local_name!("html")
    }

    #[expect(unsafe_code)]
    #[inline]
    pub(crate) fn attrs(self) -> &'dom [LayoutDom<'dom, Attr>] {
        unsafe { LayoutDom::to_layout_slice(self.unsafe_get().attrs.borrow_for_layout()) }
    }

    #[inline]
    pub(crate) fn has_class_or_part_for_layout(
        self,
        name: &AtomIdent,
        attr_name: &LocalName,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        get_attr_for_layout(self, &ns!(), attr_name).is_some_and(|attr| {
            attr.as_tokens()
                .iter()
                .any(|atom| case_sensitivity.eq_atom(atom, name))
        })
    }

    #[inline]
    pub(crate) fn get_classes_for_layout(self) -> Option<&'dom [Atom]> {
        get_attr_for_layout(self, &ns!(), &local_name!("class")).map(|attr| attr.as_tokens())
    }

    pub(crate) fn get_parts_for_layout(self) -> Option<&'dom [Atom]> {
        get_attr_for_layout(self, &ns!(), &local_name!("part")).map(|attr| attr.as_tokens())
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn style_data(self) -> Option<&'dom StyleData> {
        unsafe { self.unsafe_get().style_data.borrow_for_layout().as_deref() }
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn initialize_style_data(self) {
        let data = unsafe { self.unsafe_get().style_data.borrow_mut_for_layout() };
        debug_assert!(data.is_none());
        *data = Some(Box::default());
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) unsafe fn clear_style_data(self) {
        unsafe {
            self.unsafe_get().style_data.borrow_mut_for_layout().take();
        }
    }

    pub(crate) fn synthesize_presentational_hints_for_legacy_attributes<V>(self, hints: &mut V)
    where
        V: Push<ApplicableDeclarationBlock>,
    {
        let document = self.upcast::<Node>().owner_doc_for_layout();
        let mut property_declaration_block = None;
        let mut push = |declaration| {
            property_declaration_block
                .get_or_insert_with(PropertyDeclarationBlock::default)
                .push(declaration, Importance::Normal);
        };

        // TODO(xiaochengh): This is probably not enough. When the root element doesn't have a `lang`,
        // we should check the browser settings and system locale.
        if let Some(lang) = self.get_lang_attr_val_for_layout() {
            push(PropertyDeclaration::XLang(specified::XLang(Atom::from(
                lang.to_owned(),
            ))));
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
            push(PropertyDeclaration::BackgroundColor(
                specified::Color::from_absolute_color(color),
            ));
        }

        if is_element_affected_by_legacy_background_presentational_hint(
            self.namespace(),
            self.local_name(),
        ) {
            if let Some(url) = self
                .get_attr_for_layout(&ns!(), &local_name!("background"))
                .and_then(AttrValue::as_resolved_url)
                .cloned()
            {
                push(PropertyDeclaration::BackgroundImage(
                    background_image::SpecifiedValue(
                        vec![specified::Image::for_cascade(url)].into(),
                    ),
                ));
            }
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
            push(PropertyDeclaration::Color(
                longhands::color::SpecifiedValue(specified::Color::from_absolute_color(color)),
            ));
        }

        let font_face = self
            .downcast::<HTMLFontElement>()
            .and_then(LayoutDom::get_face);
        if let Some(font_face) = font_face {
            push(PropertyDeclaration::FontFamily(
                font_family::SpecifiedValue::Values(computed::font::FontFamilyList {
                    list: ArcSlice::from_iter(
                        HTMLFontElement::parse_face_attribute(font_face).into_iter(),
                    ),
                }),
            ));
        }

        let font_size = self
            .downcast::<HTMLFontElement>()
            .and_then(LayoutDom::get_size);
        if let Some(font_size) = font_size {
            push(PropertyDeclaration::FontSize(
                font_size::SpecifiedValue::from_html_size(font_size as u8),
            ));
        }

        // Textual input, specifically text entry and domain specific input has
        // a default preferred size.
        //
        // <https://html.spec.whatwg.org/multipage/#the-input-element-as-a-text-entry-widget>
        // <https://html.spec.whatwg.org/multipage/#the-input-element-as-domain-specific-widgets>
        let size = self
            .downcast::<HTMLInputElement>()
            .and_then(|input_element| {
                // FIXME(pcwalton): More use of atoms, please!
                match self.get_attr_val_for_layout(&ns!(), &local_name!("type")) {
                    Some("hidden") | Some("range") | Some("color") | Some("checkbox") |
                    Some("radio") | Some("file") | Some("submit") | Some("image") |
                    Some("reset") | Some("button") => None,
                    // Others
                    _ => match input_element.size_for_layout() {
                        0 => None,
                        s => Some(s as i32),
                    },
                }
            });

        if let Some(size) = size {
            let value =
                specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(size));
            push(PropertyDeclaration::Width(
                specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(value),
                )),
            ));
        }

        let width = if let Some(this) = self.downcast::<HTMLIFrameElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLImageElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLVideoElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLTableColElement>() {
            this.get_width()
        } else if let Some(this) = self.downcast::<HTMLHRElement>() {
            // https://html.spec.whatwg.org/multipage/#the-hr-element-2:attr-hr-width
            this.get_width()
        } else {
            LengthOrPercentageOrAuto::Auto
        };

        // FIXME(emilio): Use from_computed value here and below.
        match width {
            LengthOrPercentageOrAuto::Auto => {},
            LengthOrPercentageOrAuto::Percentage(percentage) => {
                let width_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Percentage(computed::Percentage(percentage)),
                ));
                push(PropertyDeclaration::Width(width_value));
            },
            LengthOrPercentageOrAuto::Length(length) => {
                let width_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(specified::NoCalcLength::Absolute(
                        specified::AbsoluteLength::Px(length.to_f32_px()),
                    )),
                ));
                push(PropertyDeclaration::Width(width_value));
            },
        }

        let height = if let Some(this) = self.downcast::<HTMLIFrameElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLImageElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLVideoElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLTableCellElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLTableRowElement>() {
            this.get_height()
        } else if let Some(this) = self.downcast::<HTMLTableSectionElement>() {
            this.get_height()
        } else {
            LengthOrPercentageOrAuto::Auto
        };

        match height {
            LengthOrPercentageOrAuto::Auto => {},
            LengthOrPercentageOrAuto::Percentage(percentage) => {
                let height_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Percentage(computed::Percentage(percentage)),
                ));
                push(PropertyDeclaration::Height(height_value));
            },
            LengthOrPercentageOrAuto::Length(length) => {
                let height_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(specified::NoCalcLength::Absolute(
                        specified::AbsoluteLength::Px(length.to_f32_px()),
                    )),
                ));
                push(PropertyDeclaration::Height(height_value));
            },
        }

        if let Some(this) = self.downcast::<SVGSVGElement>() {
            let data = this.data();
            if let Some(width) = data.width.and_then(AttrValue::as_length_percentage) {
                push(PropertyDeclaration::Width(
                    specified::Size::LengthPercentage(NonNegative(width.clone())),
                ));
            }
            if let Some(height) = data.height.and_then(AttrValue::as_length_percentage) {
                push(PropertyDeclaration::Height(
                    specified::Size::LengthPercentage(NonNegative(height.clone())),
                ));
            }
        }

        // Aspect ratio when providing both width and height.
        // https://html.spec.whatwg.org/multipage/#attributes-for-embedded-content-and-images
        if self.downcast::<HTMLImageElement>().is_some() ||
            self.downcast::<HTMLVideoElement>().is_some()
        {
            if let LengthOrPercentageOrAuto::Length(width) = width {
                if let LengthOrPercentageOrAuto::Length(height) = height {
                    let width_value = NonNegative(specified::Number::new(width.to_f32_px()));
                    let height_value = NonNegative(specified::Number::new(height.to_f32_px()));
                    let aspect_ratio = specified::position::AspectRatio {
                        auto: true,
                        ratio: PreferredRatio::Ratio(Ratio(width_value, height_value)),
                    };
                    push(PropertyDeclaration::AspectRatio(aspect_ratio));
                }
            }
        }

        let cols = self
            .downcast::<HTMLTextAreaElement>()
            .map(LayoutDom::get_cols);
        if let Some(cols) = cols {
            let cols = cols as i32;
            if cols > 0 {
                // TODO(mttr) ServoCharacterWidth uses the size math for <input type="text">, but
                // the math for <textarea> is a little different since we need to take
                // scrollbar size into consideration (but we don't have a scrollbar yet!)
                //
                // https://html.spec.whatwg.org/multipage/#textarea-effective-width
                let value =
                    specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(cols));
                push(PropertyDeclaration::Width(
                    specified::Size::LengthPercentage(NonNegative(
                        specified::LengthPercentage::Length(value),
                    )),
                ));
            }
        }

        let rows = self
            .downcast::<HTMLTextAreaElement>()
            .map(LayoutDom::get_rows);
        if let Some(rows) = rows {
            let rows = rows as i32;
            if rows > 0 {
                // TODO(mttr) This should take scrollbar size into consideration.
                //
                // https://html.spec.whatwg.org/multipage/#textarea-effective-height
                let value = specified::NoCalcLength::FontRelative(
                    specified::FontRelativeLength::Em(rows as CSSFloat),
                );
                push(PropertyDeclaration::Height(
                    specified::Size::LengthPercentage(NonNegative(
                        specified::LengthPercentage::Length(value),
                    )),
                ));
            }
        }

        if let Some(table) = self.downcast::<HTMLTableElement>() {
            if let Some(cellspacing) = table.get_cellspacing() {
                let width_value = specified::Length::from_px(cellspacing as f32);
                push(PropertyDeclaration::BorderSpacing(Box::new(
                    border_spacing::SpecifiedValue::new(
                        width_value.clone().into(),
                        width_value.into(),
                    ),
                )));
            }
            if let Some(border) = table.get_border() {
                let width_value = specified::BorderSideWidth::from_px(border as f32);
                push(PropertyDeclaration::BorderTopWidth(width_value.clone()));
                push(PropertyDeclaration::BorderLeftWidth(width_value.clone()));
                push(PropertyDeclaration::BorderBottomWidth(width_value.clone()));
                push(PropertyDeclaration::BorderRightWidth(width_value));
            }
            if document.quirks_mode() == QuirksMode::Quirks {
                // <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
                push(PropertyDeclaration::Color(color::SpecifiedValue(
                    specified::Color::InheritFromBodyQuirk,
                )));
            }
        }

        if let Some(cellpadding) = self
            .downcast::<HTMLTableCellElement>()
            .and_then(|this| this.get_table())
            .and_then(|table| table.get_cellpadding())
        {
            let cellpadding = NonNegative(specified::LengthPercentage::Length(
                specified::NoCalcLength::from_px(cellpadding as f32),
            ));
            push(PropertyDeclaration::PaddingTop(cellpadding.clone()));
            push(PropertyDeclaration::PaddingLeft(cellpadding.clone()));
            push(PropertyDeclaration::PaddingBottom(cellpadding.clone()));
            push(PropertyDeclaration::PaddingRight(cellpadding));
        }

        // https://html.spec.whatwg.org/multipage/#the-hr-element-2
        if let Some(size_info) = self
            .downcast::<HTMLHRElement>()
            .and_then(|hr_element| hr_element.get_size_info())
        {
            match size_info {
                SizePresentationalHint::SetHeightTo(height) => {
                    push(PropertyDeclaration::Height(height));
                },
                SizePresentationalHint::SetAllBorderWidthValuesTo(border_width) => {
                    push(PropertyDeclaration::BorderLeftWidth(border_width.clone()));
                    push(PropertyDeclaration::BorderRightWidth(border_width.clone()));
                    push(PropertyDeclaration::BorderTopWidth(border_width.clone()));
                    push(PropertyDeclaration::BorderBottomWidth(border_width));
                },
                SizePresentationalHint::SetBottomBorderWidthToZero => {
                    push(PropertyDeclaration::BorderBottomWidth(
                        specified::border::BorderSideWidth::from_px(0.),
                    ));
                },
            }
        }

        let Some(property_declaration_block) = property_declaration_block else {
            return;
        };

        let shared_lock = document.style_shared_lock();
        hints.push(ApplicableDeclarationBlock::from_declarations(
            ServoArc::new(shared_lock.wrap(property_declaration_block)),
            CascadeLevel::new(CascadeOrigin::PresHints),
            LayerOrder::root(),
        ));
    }

    pub(crate) fn get_span(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableColElement>()
            .and_then(|element| element.get_span())
    }

    pub(crate) fn get_colspan(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableCellElement>()
            .and_then(|element| element.get_colspan())
    }

    pub(crate) fn get_rowspan(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableCellElement>()
            .and_then(|element| element.get_rowspan())
    }

    #[inline]
    pub(crate) fn is_html_element(&self) -> bool {
        *self.namespace() == ns!(html)
    }

    #[expect(unsafe_code)]
    pub(crate) fn id_attribute(self) -> *const Option<Atom> {
        unsafe { (self.unsafe_get()).id_attribute.borrow_for_layout() }
    }

    #[expect(unsafe_code)]
    pub(crate) fn style_attribute(
        self,
    ) -> *const Option<ServoArc<Locked<PropertyDeclarationBlock>>> {
        unsafe { (self.unsafe_get()).style_attribute.borrow_for_layout() }
    }

    pub(crate) fn local_name(self) -> &'dom LocalName {
        &(self.unsafe_get()).local_name
    }

    pub(crate) fn namespace(self) -> &'dom Namespace {
        &(self.unsafe_get()).namespace
    }

    pub(crate) fn get_lang_attr_val_for_layout(self) -> Option<&'dom str> {
        if let Some(attr) = self.get_attr_val_for_layout(&ns!(xml), &local_name!("lang")) {
            return Some(attr);
        }
        if let Some(attr) = self.get_attr_val_for_layout(&ns!(), &local_name!("lang")) {
            return Some(attr);
        }
        None
    }

    pub(crate) fn get_lang_for_layout(self) -> String {
        let mut current_node = Some(self.upcast::<Node>());
        while let Some(node) = current_node {
            current_node = node.composed_parent_node_ref();
            match node.downcast::<Element>() {
                Some(elem) => {
                    if let Some(attr) = elem.get_lang_attr_val_for_layout() {
                        return attr.to_owned();
                    }
                },
                None => continue,
            }
        }
        // TODO: Check meta tags for a pragma-set default language
        // TODO: Check HTTP Content-Language header
        String::new()
    }

    #[inline]
    pub(crate) fn get_state_for_layout(self) -> ElementState {
        (self.unsafe_get()).state.get()
    }

    #[inline]
    pub(crate) fn insert_selector_flags(self, flags: ElementSelectorFlags) {
        debug_assert!(thread_state::get().is_layout());
        self.unsafe_get().insert_selector_flags(flags);
    }

    #[inline]
    pub(crate) fn get_selector_flags(self) -> ElementSelectorFlags {
        self.unsafe_get().get_selector_flags()
    }

    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn get_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>> {
        unsafe {
            self.unsafe_get()
                .rare_data
                .borrow_for_layout()
                .as_ref()?
                .shadow_root
                .as_ref()
                .map(|sr| sr.to_layout())
        }
    }

    #[inline]
    pub(crate) fn get_attr_for_layout(
        self,
        namespace: &Namespace,
        name: &LocalName,
    ) -> Option<&'dom AttrValue> {
        get_attr_for_layout(self, namespace, name)
    }

    #[inline]
    pub(crate) fn get_attr_val_for_layout(
        self,
        namespace: &Namespace,
        name: &LocalName,
    ) -> Option<&'dom str> {
        get_attr_for_layout(self, namespace, name).map(|attr| &**attr)
    }

    #[inline]
    pub(crate) fn get_attr_vals_for_layout(
        self,
        name: &LocalName,
    ) -> impl Iterator<Item = &'dom AttrValue> {
        self.attrs().iter().filter_map(move |attr| {
            if name == attr.local_name() {
                Some(attr.value())
            } else {
                None
            }
        })
    }

    #[expect(unsafe_code)]
    pub(crate) fn each_custom_state_for_layout(self, mut callback: impl FnMut(&AtomIdent)) {
        let rare_data = unsafe { self.unsafe_get().rare_data.borrow_for_layout() };
        let Some(rare_data) = rare_data.as_ref() else {
            return;
        };
        let Some(element_internals) = rare_data.element_internals.as_ref() else {
            return;
        };

        let element_internals = unsafe { element_internals.to_layout() };
        if let Some(states) = element_internals.unsafe_get().custom_states_for_layout() {
            for state in unsafe { states.unsafe_get().set_for_layout().iter() } {
                // FIXME: This creates new atoms whenever it is called, which is not optimal.
                callback(&AtomIdent::from(&*state.str()));
            }
        }
    }
}

impl Element {
    pub(crate) fn is_html_element(&self) -> bool {
        self.namespace == ns!(html)
    }

    pub(crate) fn html_element_in_html_document(&self) -> bool {
        self.is_html_element() && self.upcast::<Node>().is_in_html_doc()
    }

    pub(crate) fn local_name(&self) -> &LocalName {
        &self.local_name
    }

    pub(crate) fn parsed_name(&self, mut name: DOMString) -> LocalName {
        if self.html_element_in_html_document() {
            name.make_ascii_lowercase();
        }
        LocalName::from(name)
    }

    pub(crate) fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    pub(crate) fn prefix(&self) -> Ref<'_, Option<Prefix>> {
        self.prefix.borrow()
    }

    pub(crate) fn set_prefix(&self, prefix: Option<Prefix>) {
        *self.prefix.borrow_mut() = prefix;
    }

    pub(crate) fn set_custom_element_registry(
        &self,
        registry: Option<DomRoot<CustomElementRegistry>>,
    ) {
        self.ensure_rare_data().custom_element_registry = registry.as_deref().map(Dom::from_ref);
    }

    pub(crate) fn custom_element_registry(&self) -> Option<DomRoot<CustomElementRegistry>> {
        self.rare_data()
            .as_ref()?
            .custom_element_registry
            .as_deref()
            .map(DomRoot::from_ref)
    }

    pub(crate) fn attrs(&self) -> Ref<'_, [Dom<Attr>]> {
        Ref::map(self.attrs.borrow(), |attrs| &**attrs)
    }

    /// Element branch of <https://dom.spec.whatwg.org/#locate-a-namespace>
    pub(crate) fn locate_namespace(&self, prefix: Option<DOMString>) -> Namespace {
        let namespace_prefix = prefix.clone().map(|s| Prefix::from(&*s.str()));

        // Step 1. If prefix is "xml", then return the XML namespace.
        if namespace_prefix == Some(namespace_prefix!("xml")) {
            return ns!(xml);
        }

        // Step 2. If prefix is "xmlns", then return the XMLNS namespace.
        if namespace_prefix == Some(namespace_prefix!("xmlns")) {
            return ns!(xmlns);
        }

        let prefix = prefix.map(LocalName::from);

        let inclusive_ancestor_elements = self
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Self>);

        // Step 5. If its parent element is null, then return null.
        // Step 6. Return the result of running locate a namespace on its parent element using prefix.
        for element in inclusive_ancestor_elements {
            // Step 3. If its namespace is non-null and its namespace prefix is prefix, then return namespace.
            if element.namespace() != &ns!() &&
                element.prefix().as_ref().map(|p| &**p) == prefix.as_deref()
            {
                return element.namespace().clone();
            }

            // Step 4. If it has an attribute whose namespace is the XMLNS namespace, namespace prefix
            // is "xmlns", and local name is prefix, or if prefix is null and it has an attribute
            // whose namespace is the XMLNS namespace, namespace prefix is null, and local name is
            // "xmlns", then return its value if it is not the empty string, and null otherwise.
            let attr = Ref::filter_map(self.attrs(), |attrs| {
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
            })
            .ok();

            if let Some(attr) = attr {
                return (**attr.value()).into();
            }
        }

        ns!()
    }

    pub(crate) fn name_attribute(&self) -> Option<Atom> {
        self.rare_data().as_ref()?.name_attribute.clone()
    }

    pub(crate) fn style_attribute(
        &self,
    ) -> &DomRefCell<Option<ServoArc<Locked<PropertyDeclarationBlock>>>> {
        &self.style_attribute
    }

    pub(crate) fn summarize(&self) -> Vec<AttrInfo> {
        self.attrs
            .borrow()
            .iter()
            .map(|attr| attr.summarize())
            .collect()
    }

    pub(crate) fn is_void(&self) -> bool {
        if self.namespace != ns!(html) {
            return false;
        }
        match self.local_name {
            /* List of void elements from
            https://html.spec.whatwg.org/multipage/#html-fragment-serialisation-algorithm */
            local_name!("area") |
            local_name!("base") |
            local_name!("basefont") |
            local_name!("bgsound") |
            local_name!("br") |
            local_name!("col") |
            local_name!("embed") |
            local_name!("frame") |
            local_name!("hr") |
            local_name!("img") |
            local_name!("input") |
            local_name!("keygen") |
            local_name!("link") |
            local_name!("meta") |
            local_name!("param") |
            local_name!("source") |
            local_name!("track") |
            local_name!("wbr") => true,
            _ => false,
        }
    }

    pub(crate) fn root_element(&self) -> DomRoot<Element> {
        if self.node.is_in_a_document_tree() {
            self.upcast::<Node>()
                .owner_doc()
                .GetDocumentElement()
                .unwrap()
        } else {
            self.upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::No)
                .filter_map(DomRoot::downcast)
                .last()
                .expect("We know inclusive_ancestors will return `self` which is an element")
        }
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace-prefix
    pub(crate) fn lookup_prefix(&self, namespace: Namespace) -> Option<DOMString> {
        for node in self
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::No)
        {
            let element = node.downcast::<Element>()?;
            // Step 1.
            if *element.namespace() == namespace {
                if let Some(prefix) = element.GetPrefix() {
                    return Some(prefix);
                }
            }

            // Step 2.
            for attr in element.attrs.borrow().iter() {
                if attr.prefix() == Some(&namespace_prefix!("xmlns")) &&
                    **attr.value() == *namespace
                {
                    return Some(attr.LocalName());
                }
            }
        }
        None
    }

    /// <https://dom.spec.whatwg.org/#document-element>
    pub(crate) fn is_document_element(&self) -> bool {
        if let Some(document_element) = self.owner_document().GetDocumentElement() {
            *document_element == *self
        } else {
            false
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-document-activeelement>
    pub(crate) fn is_active_element(&self) -> bool {
        if let Some(active_element) = self.owner_document().GetActiveElement() {
            *active_element == *self
        } else {
            false
        }
    }

    pub(crate) fn is_editing_host(&self) -> bool {
        self.downcast::<HTMLElement>()
            .is_some_and(|element| element.IsContentEditable())
    }

    pub(crate) fn is_actually_disabled(&self) -> bool {
        let node = self.upcast::<Node>();
        match node.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLTextAreaElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLOptionElement,
            )) => self.disabled_state(),
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLElement)) => {
                self.downcast::<HTMLElement>()
                    .unwrap()
                    .is_form_associated_custom_element() &&
                    self.disabled_state()
            },
            // TODO:
            // an optgroup element that has a disabled attribute
            // a menuitem element that has a disabled attribute
            // a fieldset element that is a disabled fieldset
            _ => false,
        }
    }

    #[expect(unsafe_code)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn push_new_attribute(
        &self,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        reason: AttributeMutationReason,
        can_gc: CanGc,
    ) {
        // TODO: https://github.com/servo/servo/issues/42812
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        let cx = &mut cx;
        let attr = Attr::new(
            cx,
            &self.node.owner_doc(),
            local_name,
            value,
            name,
            namespace,
            prefix,
            Some(self),
        );
        self.push_attribute(&attr, reason, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#handle-attribute-changes>
    #[expect(unsafe_code)]
    fn handle_attribute_changes(
        &self,
        attr: &Attr,
        old_value: Option<&AttrValue>,
        new_value: Option<DOMString>,
        reason: AttributeMutationReason,
        _can_gc: CanGc,
    ) {
        // TODO: https://github.com/servo/servo/issues/42812
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        let cx = &mut cx;

        let old_value_string = old_value.map(|old_value| DOMString::from(&**old_value));
        // Step 1. Queue a mutation record of "attributes" for element with attribute’s local name,
        // attribute’s namespace, oldValue, « », « », null, and null.
        let name = attr.local_name().clone();
        let namespace = attr.namespace().clone();
        let mutation = LazyCell::new(|| Mutation::Attribute {
            name: name.clone(),
            namespace: namespace.clone(),
            old_value: old_value_string.clone(),
        });
        MutationObserver::queue_a_mutation_record(&self.node, mutation);

        // Avoid double borrow
        let has_new_value = new_value.is_some();

        // Step 2. If element is custom, then enqueue a custom element callback reaction with element,
        // callback name "attributeChangedCallback", and « attribute’s local name, oldValue, newValue, attribute’s namespace ».
        if self.is_custom() {
            let reaction = CallbackReaction::AttributeChanged(
                attr.local_name().clone(),
                old_value_string,
                new_value,
                attr.namespace().clone(),
            );
            ScriptThread::enqueue_callback_reaction(self, reaction, None);
        }

        // Step 3. Run the attribute change steps with element, attribute’s local name, oldValue, newValue, and attribute’s namespace.
        if is_relevant_attribute(attr.namespace(), attr.local_name()) {
            let attribute_mutation = if has_new_value {
                AttributeMutation::Set(old_value, reason)
            } else {
                AttributeMutation::Removed
            };
            vtable_for(self.upcast()).attribute_mutated(cx, attr, attribute_mutation);
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-change>
    pub(crate) fn change_attribute(&self, attr: &Attr, mut value: AttrValue, can_gc: CanGc) {
        // Step 1. Let oldValue be attribute’s value.
        //
        // Clone to avoid double borrow
        let old_value = &attr.value().clone();
        // Step 2. Set attribute’s value to value.
        self.will_mutate_attr(attr);
        attr.swap_value(&mut value);
        // Step 3. Handle attribute changes for attribute with attribute’s element, oldValue, and value.
        //
        // Put on a separate line to avoid double borrow
        let new_value = DOMString::from(&**attr.value());
        self.handle_attribute_changes(
            attr,
            Some(old_value),
            Some(new_value),
            AttributeMutationReason::Directly,
            can_gc,
        );
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-append>
    pub(crate) fn push_attribute(
        &self,
        attr: &Attr,
        reason: AttributeMutationReason,
        can_gc: CanGc,
    ) {
        // Step 2. Set attribute’s element to element.
        //
        // Handled by callers of this function and asserted here.
        assert!(attr.GetOwnerElement().as_deref() == Some(self));
        // Step 3. Set attribute’s node document to element’s node document.
        //
        // Handled by callers of this function and asserted here.
        assert!(attr.upcast::<Node>().owner_doc() == self.node.owner_doc());
        // Step 1. Append attribute to element’s attribute list.
        self.will_mutate_attr(attr);
        self.attrs.borrow_mut().push(Dom::from_ref(attr));
        // Step 4. Handle attribute changes for attribute with element, null, and attribute’s value.
        //
        // Put on a separate line to avoid double borrow
        let new_value = DOMString::from(&**attr.value());
        self.handle_attribute_changes(attr, None, Some(new_value), reason, can_gc);
    }

    /// This is the inner logic for:
    /// <https://dom.spec.whatwg.org/#concept-element-attributes-get-by-namespace>
    ///
    /// In addition to taking a namespace argument, this version does not require the attribute
    /// to be lowercase ASCII, in accordance with the specification.
    pub(crate) fn get_attribute_with_namespace(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
    ) -> Option<DomRoot<Attr>> {
        self.attrs
            .borrow()
            .iter()
            .find(|attr| attr.local_name() == local_name && attr.namespace() == namespace)
            .map(|js| DomRoot::from_ref(&**js))
    }

    /// This is the inner logic for:
    /// <https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name>
    ///
    /// Callers should convert the `LocalName` to ASCII lowercase before calling.
    pub(crate) fn get_attribute(&self, local_name: &LocalName) -> Option<DomRoot<Attr>> {
        debug_assert_eq!(
            *local_name,
            local_name.to_ascii_lowercase(),
            "All namespace-less attribute accesses should use a lowercase ASCII name"
        );
        self.get_attribute_with_namespace(&ns!(), local_name)
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-get-by-name>
    pub(crate) fn get_attribute_by_name(&self, name: DOMString) -> Option<DomRoot<Attr>> {
        let name = &self.parsed_name(name);
        let maybe_attribute = self
            .attrs
            .borrow()
            .iter()
            .find(|a| a.name() == name)
            .map(|js| DomRoot::from_ref(&**js));
        fn id_and_name_must_be_atoms(name: &LocalName, maybe_attr: &Option<DomRoot<Attr>>) -> bool {
            if *name == local_name!("id") || *name == local_name!("name") {
                match maybe_attr {
                    None => true,
                    Some(attr) => matches!(*attr.value(), AttrValue::Atom(_)),
                }
            } else {
                true
            }
        }
        debug_assert!(id_and_name_must_be_atoms(name, &maybe_attribute));
        maybe_attribute
    }

    pub(crate) fn set_attribute_from_parser(
        &self,
        qname: QualName,
        value: DOMString,
        prefix: Option<Prefix>,
        can_gc: CanGc,
    ) {
        // Don't set if the attribute already exists, so we can handle add_attrs_if_missing
        if self
            .attrs
            .borrow()
            .iter()
            .any(|a| *a.local_name() == qname.local && *a.namespace() == qname.ns)
        {
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
        self.push_new_attribute(
            qname.local,
            value,
            name,
            qname.ns,
            prefix,
            AttributeMutationReason::ByParser,
            can_gc,
        );
    }

    pub(crate) fn set_attribute(&self, name: &LocalName, value: AttrValue, can_gc: CanGc) {
        debug_assert_eq!(
            *name,
            name.to_ascii_lowercase(),
            "All attribute accesses should use a lowercase ASCII name"
        );
        debug_assert!(!name.contains(':'));

        self.set_first_matching_attribute(
            name.clone(),
            value,
            name.clone(),
            ns!(),
            None,
            |attr| attr.local_name() == name,
            can_gc,
        );
    }

    pub(crate) fn set_attribute_with_namespace(
        &self,
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
    ) {
        self.set_first_matching_attribute(
            local_name.clone(),
            value,
            name,
            namespace.clone(),
            prefix,
            |attr| *attr.local_name() == local_name && *attr.namespace() == namespace,
            CanGc::from_cx(cx),
        );
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-set-value>
    #[allow(clippy::too_many_arguments)]
    fn set_first_matching_attribute<F>(
        &self,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        find: F,
        can_gc: CanGc,
    ) where
        F: Fn(&Attr) -> bool,
    {
        // Step 1. Let attribute be the result of getting an attribute given namespace, localName, and element.
        let attr = self
            .attrs
            .borrow()
            .iter()
            .find(|attr| find(attr))
            .map(|js| DomRoot::from_ref(&**js));
        if let Some(attr) = attr {
            // Step 3. Change attribute to value.
            self.will_mutate_attr(&attr);
            self.change_attribute(&attr, value, can_gc);
        } else {
            // Step 2. If attribute is null, create an attribute whose namespace is namespace,
            // namespace prefix is prefix, local name is localName, value is value,
            // and node document is element’s node document,
            // then append this attribute to element, and then return.
            self.push_new_attribute(
                local_name,
                value,
                name,
                namespace,
                prefix,
                AttributeMutationReason::Directly,
                can_gc,
            );
        };
    }

    pub(crate) fn parse_attribute(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
        value: DOMString,
    ) -> AttrValue {
        if is_relevant_attribute(namespace, local_name) {
            vtable_for(self.upcast()).parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value.into())
        }
    }

    pub(crate) fn remove_attribute(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
        can_gc: CanGc,
    ) -> Option<DomRoot<Attr>> {
        self.remove_first_matching_attribute(
            |attr| attr.namespace() == namespace && attr.local_name() == local_name,
            can_gc,
        )
    }

    pub(crate) fn remove_attribute_by_name(
        &self,
        name: &LocalName,
        can_gc: CanGc,
    ) -> Option<DomRoot<Attr>> {
        self.remove_first_matching_attribute(|attr| attr.name() == name, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-remove>
    fn remove_first_matching_attribute<F>(&self, find: F, can_gc: CanGc) -> Option<DomRoot<Attr>>
    where
        F: Fn(&Attr) -> bool,
    {
        let idx = self.attrs.borrow().iter().position(|attr| find(attr));
        idx.map(|idx| {
            let attr = DomRoot::from_ref(&*(*self.attrs.borrow())[idx]);

            // Step 2. Remove attribute from element’s attribute list.
            self.will_mutate_attr(&attr);
            self.attrs.borrow_mut().remove(idx);
            // Step 3. Set attribute’s element to null.
            attr.set_owner(None);
            // Step 4. Handle attribute changes for attribute with element, attribute’s value, and null.
            self.handle_attribute_changes(
                &attr,
                Some(&attr.value()),
                None,
                AttributeMutationReason::Directly,
                can_gc,
            );

            attr
        })
    }

    pub(crate) fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        self.get_attribute(&local_name!("class"))
            .is_some_and(|attr| {
                attr.value()
                    .as_tokens()
                    .iter()
                    .any(|atom| case_sensitivity.eq_atom(name, atom))
            })
    }

    pub(crate) fn has_attribute(&self, local_name: &LocalName) -> bool {
        debug_assert_eq!(
            *local_name,
            local_name.to_ascii_lowercase(),
            "All attribute accesses should use a lowercase ASCII name"
        );
        debug_assert!(!local_name.contains(':'));
        self.attrs
            .borrow()
            .iter()
            .any(|attr| attr.local_name() == local_name && attr.namespace() == &ns!())
    }

    pub(crate) fn will_mutate_attr(&self, attr: &Attr) {
        let node = self.upcast::<Node>();
        node.owner_doc().element_attr_will_change(self, attr);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-style-attribute>
    fn update_style_attribute(&self, attr: &Attr, mutation: AttributeMutation) {
        let doc = self.upcast::<Node>().owner_doc();
        // Modifying the `style` attribute might change style.
        *self.style_attribute.borrow_mut() = match mutation {
            AttributeMutation::Set(..) => {
                // This is the fast path we use from
                // CSSStyleDeclaration.
                //
                // Juggle a bit to keep the borrow checker happy
                // while avoiding the extra clone.
                let is_declaration = matches!(*attr.value(), AttrValue::Declaration(..));

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
                    let win = self.owner_window();
                    let source = &**attr.value();
                    let global = &self.owner_global();
                    // However, if the Should element's inline behavior be blocked by
                    // Content Security Policy? algorithm returns "Blocked" when executed
                    // upon the attribute's element, "style attribute", and the attribute's value,
                    // then the style rules defined in the attribute's value must not be applied to the element. [CSP]
                    if global
                        .get_csp_list()
                        .should_elements_inline_type_behavior_be_blocked(
                            global,
                            self,
                            InlineCheckType::StyleAttribute,
                            source,
                            doc.get_current_parser_line(),
                        )
                    {
                        return;
                    }
                    ServoArc::new(doc.style_shared_lock().wrap(parse_style_attribute(
                        source,
                        &UrlExtraData(doc.base_url().get_arc()),
                        Some(win.css_error_reporter()),
                        doc.quirks_mode(),
                        CssRuleType::Style,
                    )))
                };

                Some(block)
            },
            AttributeMutation::Removed => None,
        };
    }

    /// <https://dom.spec.whatwg.org/#concept-element-attributes-set>
    /// including steps of
    /// <https://dom.spec.whatwg.org/#concept-element-attributes-replace>
    fn set_attribute_node(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<Option<DomRoot<Attr>>> {
        // Step 1. Let verifiedValue be the result of calling
        // get Trusted Types-compliant attribute value with attr’s local name,
        // attr’s namespace, element, and attr’s value. [TRUSTED-TYPES]
        let verified_value = TrustedTypePolicyFactory::get_trusted_types_compliant_attribute_value(
            cx,
            self.namespace(),
            self.local_name(),
            attr.local_name(),
            Some(attr.namespace()),
            TrustedTypeOrString::String(attr.Value()),
            &self.owner_global(),
        )?;

        // Step 2. If attr’s element is neither null nor element,
        // throw an "InUseAttributeError" DOMException.
        if let Some(owner) = attr.GetOwnerElement() {
            if &*owner != self {
                return Err(Error::InUseAttribute(None));
            }
        }

        let vtable = vtable_for(self.upcast());

        // Step 5. Set attr’s value to verifiedValue.
        //
        // This ensures that the attribute is of the expected kind for this
        // specific element. This is inefficient and should probably be done
        // differently.
        attr.swap_value(
            &mut vtable.parse_plain_attribute(attr.local_name(), verified_value.clone()),
        );

        // Step 3. Let oldAttr be the result of getting an attribute given attr’s namespace, attr’s local name, and element.
        let position = self.attrs.borrow().iter().position(|old_attr| {
            attr.namespace() == old_attr.namespace() && attr.local_name() == old_attr.local_name()
        });

        let old_attr = if let Some(position) = position {
            let old_attr = DomRoot::from_ref(&*self.attrs.borrow()[position]);

            // Step 4. If oldAttr is attr, return attr.
            if &*old_attr == attr {
                return Ok(Some(DomRoot::from_ref(attr)));
            }

            // Step 6. If oldAttr is non-null, then replace oldAttr with attr.
            //
            // Start of steps for https://dom.spec.whatwg.org/#concept-element-attributes-replace

            // Step 1. Let element be oldAttribute’s element.
            //
            // Skipped, as that points to self.

            // Step 2. Replace oldAttribute by newAttribute in element’s attribute list.
            self.will_mutate_attr(attr);
            self.attrs.borrow_mut()[position] = Dom::from_ref(attr);
            // Step 3. Set newAttribute’s element to element.
            attr.set_owner(Some(self));
            // Step 4. Set newAttribute’s node document to element’s node document.
            attr.upcast::<Node>().set_owner_doc(&self.node.owner_doc());
            // Step 5. Set oldAttribute’s element to null.
            old_attr.set_owner(None);
            // Step 6. Handle attribute changes for oldAttribute with element, oldAttribute’s value, and newAttribute’s value.
            self.handle_attribute_changes(
                attr,
                Some(&old_attr.value()),
                Some(verified_value),
                AttributeMutationReason::Directly,
                CanGc::from_cx(cx),
            );

            Some(old_attr)
        } else {
            // Step 7. Otherwise, append attr to element.
            attr.set_owner(Some(self));
            attr.upcast::<Node>().set_owner_doc(&self.node.owner_doc());
            self.push_attribute(attr, AttributeMutationReason::Directly, CanGc::from_cx(cx));

            None
        };

        // Step 8. Return oldAttr.
        Ok(old_attr)
    }

    /// <https://html.spec.whatwg.org/multipage/#nonce-attributes>
    pub(crate) fn update_nonce_internal_slot(&self, nonce: String) {
        self.ensure_rare_data().cryptographic_nonce = nonce;
    }

    /// <https://html.spec.whatwg.org/multipage/#nonce-attributes>
    pub(crate) fn nonce_value(&self) -> String {
        match self.rare_data().as_ref() {
            None => String::new(),
            Some(rare_data) => rare_data.cryptographic_nonce.clone(),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#nonce-attributes>
    pub(crate) fn update_nonce_post_connection(&self) {
        // Whenever an element including HTMLOrSVGElement becomes browsing-context connected,
        // the user agent must execute the following steps on the element:
        if !self.upcast::<Node>().is_connected_with_browsing_context() {
            return;
        }
        let global = self.owner_global();
        // Step 1: Let CSP list be element's shadow-including root's policy container's CSP list.
        let csp_list = match global.get_csp_list() {
            None => return,
            Some(csp_list) => csp_list,
        };
        // Step 2: If CSP list contains a header-delivered Content Security Policy,
        // and element has a nonce content attribute whose value is not the empty string, then:
        if !csp_list.contains_a_header_delivered_content_security_policy() ||
            self.get_string_attribute(&local_name!("nonce")).is_empty()
        {
            return;
        }
        // Step 2.1: Let nonce be element's [[CryptographicNonce]].
        let nonce = self.nonce_value();
        // Step 2.2: Set an attribute value for element using "nonce" and the empty string.
        self.set_string_attribute(&local_name!("nonce"), "".into(), CanGc::deprecated_note());
        // Step 2.3: Set element's [[CryptographicNonce]] to nonce.
        self.update_nonce_internal_slot(nonce);
    }

    /// <https://www.w3.org/TR/CSP/#is-element-nonceable>
    pub(crate) fn is_nonceable(&self) -> bool {
        // Step 1: If element does not have an attribute named "nonce", return "Not Nonceable".
        if !self.has_attribute(&local_name!("nonce")) {
            return false;
        }
        // Step 2: If element is a script element, then for each attribute of element’s attribute list:
        if self.downcast::<HTMLScriptElement>().is_some() {
            for attr in self.attrs().iter() {
                // Step 2.1: If attribute’s name contains an ASCII case-insensitive match
                // for "<script" or "<style", return "Not Nonceable".
                let attr_name = attr.name().to_ascii_lowercase();
                if attr_name.contains("<script") || attr_name.contains("<style") {
                    return false;
                }
                // Step 2.2: If attribute’s value contains an ASCII case-insensitive match
                // for "<script" or "<style", return "Not Nonceable".
                let attr_value = attr.value().to_ascii_lowercase();
                if attr_value.contains("<script") || attr_value.contains("<style") {
                    return false;
                }
            }
        }
        // Step 3: If element had a duplicate-attribute parse error during tokenization, return "Not Nonceable".
        if self
            .rare_data()
            .as_ref()
            .is_some_and(|d| d.had_duplicate_attributes)
        {
            return false;
        }
        // Step 4: Return "Nonceable".
        true
    }

    // https://dom.spec.whatwg.org/#insert-adjacent
    pub(crate) fn insert_adjacent(
        &self,
        cx: &mut JSContext,
        where_: AdjacentPosition,
        node: &Node,
    ) -> Fallible<Option<DomRoot<Node>>> {
        let self_node = self.upcast::<Node>();
        match where_ {
            AdjacentPosition::BeforeBegin => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(cx, node, &parent, Some(self_node)).map(Some)
                } else {
                    Ok(None)
                }
            },
            AdjacentPosition::AfterBegin => {
                Node::pre_insert(cx, node, self_node, self_node.GetFirstChild().as_deref())
                    .map(Some)
            },
            AdjacentPosition::BeforeEnd => Node::pre_insert(cx, node, self_node, None).map(Some),
            AdjacentPosition::AfterEnd => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(cx, node, &parent, self_node.GetNextSibling().as_deref())
                        .map(Some)
                } else {
                    Ok(None)
                }
            },
        }
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scroll>
    ///
    /// TODO(stevennovaryo): Need to update the scroll API to follow the spec since it is
    /// quite outdated.
    pub(crate) fn scroll(&self, x: f64, y: f64, behavior: ScrollBehavior) {
        // Step 1.2 or 2.3
        let x = if x.is_finite() { x } else { 0.0 } as f32;
        let y = if y.is_finite() { y } else { 0.0 } as f32;

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
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body()
        {
            win.scroll(x, y, behavior);
            return;
        }

        // Step 10
        if !self.has_scrolling_box() {
            return;
        }

        // Step 11
        win.scroll_an_element(self, x, y, behavior);
    }

    /// <https://html.spec.whatwg.org/multipage/#fragment-parsing-algorithm-steps>
    pub(crate) fn parse_fragment(
        &self,
        markup: DOMString,
        cx: &mut js::context::JSContext,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        // Steps 1-2.
        // TODO(#11995): XML case.
        let new_children = ServoParser::parse_html_fragment(self, markup, false, cx);
        // Step 3.
        // See https://github.com/w3c/DOM-Parsing/issues/61.
        let context_document = {
            if let Some(template) = self.downcast::<HTMLTemplateElement>() {
                template.Content(cx).upcast::<Node>().owner_doc()
            } else {
                self.owner_document()
            }
        };
        let fragment = DocumentFragment::new(cx, &context_document);
        // Step 4.
        for child in new_children {
            fragment.upcast::<Node>().AppendChild(cx, &child).unwrap();
        }
        // Step 5.
        Ok(fragment)
    }

    /// Step 4 of <https://html.spec.whatwg.org/multipage/#dom-element-insertadjacenthtml>
    /// and step 6. of <https://html.spec.whatwg.org/multipage/#dom-range-createcontextualfragment>
    pub(crate) fn fragment_parsing_context(
        cx: &mut JSContext,
        owner_doc: &Document,
        element: Option<&Self>,
    ) -> DomRoot<Self> {
        // If context is not an Element or all of the following are true:
        match element {
            Some(elem)
                // context's node document is an HTML document;
                // context's local name is "html"; and
                // context's namespace is the HTML namespace,
                if elem.local_name() != &local_name!("html") ||
                    !elem.html_element_in_html_document() =>
            {
                DomRoot::from_ref(elem)
            },
            // set context to the result of creating an element
            // given this's node document, "body", and the HTML namespace.
            _ => Element::create(
                cx,
                QualName::new(None, ns!(html), local_name!("body")),
                None,
                owner_doc,
                ElementCreator::ScriptCreated,
                CustomElementCreationMode::Asynchronous,
                None
            ),
        }
    }

    // https://html.spec.whatwg.org/multipage/#home-subtree
    pub(crate) fn is_in_same_home_subtree<T>(&self, other: &T) -> bool
    where
        T: DerivedFrom<Element> + DomObject,
    {
        let other = other.upcast::<Element>();
        self.root_element() == other.root_element()
    }

    pub(crate) fn get_id(&self) -> Option<Atom> {
        self.id_attribute.borrow().clone()
    }

    pub(crate) fn get_name(&self) -> Option<Atom> {
        self.rare_data().as_ref()?.name_attribute.clone()
    }

    pub(crate) fn get_element_internals(&self) -> Option<DomRoot<ElementInternals>> {
        self.rare_data()
            .as_ref()?
            .element_internals
            .as_ref()
            .map(|sr| DomRoot::from_ref(&**sr))
    }

    pub(crate) fn ensure_element_internals(&self, can_gc: CanGc) -> DomRoot<ElementInternals> {
        let mut rare_data = self.ensure_rare_data();
        DomRoot::from_ref(rare_data.element_internals.get_or_insert_with(|| {
            let elem = self
                .downcast::<HTMLElement>()
                .expect("ensure_element_internals should only be called for an HTMLElement");
            Dom::from_ref(&*ElementInternals::new(elem, can_gc))
        }))
    }

    pub(crate) fn outer_html(&self, cx: &mut js::context::JSContext) -> Fallible<DOMString> {
        match self.GetOuterHTML(cx)? {
            TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(str) => Ok(str),
            TrustedHTMLOrNullIsEmptyString::TrustedHTML(_) => unreachable!(),
        }
    }

    pub(crate) fn compute_source_position(&self, line_number: u32) -> SourcePosition {
        SourcePosition {
            source_file: self.owner_global().get_url().to_string(),
            line_number: line_number + 2,
            column_number: 0,
        }
    }

    pub(crate) fn explicitly_set_tab_index(&self) -> Option<i32> {
        if self.has_attribute(&local_name!("tabindex")) {
            Some(self.get_int_attribute(&local_name!("tabindex"), 0))
        } else {
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    pub(crate) fn tab_index(&self) -> i32 {
        // > The tabIndex getter steps are:
        // > 1. Let attribute be this's tabindex attribute.
        // > 2. If attribute is not null:
        // >    1. Let parsedValue be the result of integer parsing attribute's value.
        // >    2. If parsedValue is not an error and is within the long range, then return parsedValue.
        if let Some(tab_index) = self.explicitly_set_tab_index() {
            return tab_index;
        }

        // > 3. Return 0 if this is an a, area, button, frame, iframe, input, object, select, textarea,
        // > or SVG a element, or is a summary element that is a summary for its parent details;
        // > otherwise -1.
        //
        // Note: We do not currently support SVG `a` elements.
        if matches!(
            self.upcast::<Node>().type_id(),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement |
                    HTMLElementTypeId::HTMLAreaElement |
                    HTMLElementTypeId::HTMLButtonElement |
                    HTMLElementTypeId::HTMLFrameElement |
                    HTMLElementTypeId::HTMLIFrameElement |
                    HTMLElementTypeId::HTMLInputElement |
                    HTMLElementTypeId::HTMLObjectElement |
                    HTMLElementTypeId::HTMLSelectElement |
                    HTMLElementTypeId::HTMLTextAreaElement
            ))
        ) {
            return 0;
        }
        if self
            .downcast::<HTMLElement>()
            .is_some_and(|html_element| html_element.is_a_summary_for_its_parent_details())
        {
            return 0;
        }

        -1
    }

    #[inline]
    fn insert_selector_flags(&self, flags: ElementSelectorFlags) {
        self.selector_flags
            .fetch_or(flags.bits(), Ordering::Relaxed);
    }

    #[inline]
    fn get_selector_flags(&self) -> ElementSelectorFlags {
        ElementSelectorFlags::from_bits_retain(self.selector_flags.load(Ordering::Relaxed))
    }
}

impl ElementMethods<crate::DomTypeHolder> for Element {
    /// <https://dom.spec.whatwg.org/#dom-element-namespaceuri>
    fn GetNamespaceURI(&self) -> Option<DOMString> {
        Node::namespace_to_string(self.namespace.clone())
    }

    /// <https://dom.spec.whatwg.org/#dom-element-localname>
    fn LocalName(&self) -> DOMString {
        // FIXME(ajeffrey): Convert directly from LocalName to DOMString
        DOMString::from(&*self.local_name)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-prefix>
    fn GetPrefix(&self) -> Option<DOMString> {
        self.prefix.borrow().as_ref().map(|p| DOMString::from(&**p))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-tagname>
    fn TagName(&self) -> DOMString {
        let name = self.tag_name.or_init(|| {
            let qualified_name = match *self.prefix.borrow() {
                Some(ref prefix) => Cow::Owned(format!("{}:{}", &**prefix, &*self.local_name)),
                None => Cow::Borrowed(&*self.local_name),
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
    // This always returns a string; if you'd rather see None
    // on a null id, call get_id
    fn Id(&self) -> DOMString {
        self.get_string_attribute(&local_name!("id"))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-id>
    fn SetId(&self, id: DOMString, can_gc: CanGc) {
        self.set_atomic_attribute(&local_name!("id"), id, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-element-classname>
    fn ClassName(&self) -> DOMString {
        self.get_string_attribute(&local_name!("class"))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-classname>
    fn SetClassName(&self, class: DOMString, can_gc: CanGc) {
        self.set_tokenlist_attribute(&local_name!("class"), class, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-element-classlist>
    fn ClassList(&self, can_gc: CanGc) -> DomRoot<DOMTokenList> {
        self.class_list
            .or_init(|| DOMTokenList::new(self, &local_name!("class"), None, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_getter!(Slot, "slot");

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_setter!(SetSlot, "slot");

    /// <https://dom.spec.whatwg.org/#dom-element-attributes>
    fn Attributes(&self, can_gc: CanGc) -> DomRoot<NamedNodeMap> {
        self.attr_list
            .or_init(|| NamedNodeMap::new(&self.owner_window(), self, can_gc))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-hasattributes>
    fn HasAttributes(&self) -> bool {
        !self.attrs.borrow().is_empty()
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getattributenames>
    fn GetAttributeNames(&self) -> Vec<DOMString> {
        self.attrs.borrow().iter().map(|attr| attr.Name()).collect()
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getattribute>
    fn GetAttribute(&self, name: DOMString) -> Option<DOMString> {
        self.GetAttributeNode(name).map(|s| s.Value())
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getattributens>
    fn GetAttributeNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Option<DOMString> {
        self.GetAttributeNodeNS(namespace, local_name)
            .map(|attr| attr.Value())
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getattributenode>
    fn GetAttributeNode(&self, name: DOMString) -> Option<DomRoot<Attr>> {
        self.get_attribute_by_name(name)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getattributenodens>
    fn GetAttributeNodeNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Option<DomRoot<Attr>> {
        let namespace = &namespace_from_domstring(namespace);
        self.get_attribute_with_namespace(namespace, &LocalName::from(local_name))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-toggleattribute>
    fn ToggleAttribute(
        &self,
        cx: &mut js::context::JSContext,
        name: DOMString,
        force: Option<bool>,
    ) -> Fallible<bool> {
        // Step 1. If qualifiedName is not a valid attribute local name,
        //      then throw an "InvalidCharacterError" DOMException.
        if !is_valid_attribute_local_name(&name.str()) {
            return Err(Error::InvalidCharacter(None));
        }

        // Step 3.
        let attribute = self.GetAttribute(name.clone());

        // Step 2.
        let name = self.parsed_name(name);
        match attribute {
            // Step 4
            None => match force {
                // Step 4.1.
                None | Some(true) => {
                    self.set_first_matching_attribute(
                        name.clone(),
                        AttrValue::String(String::new()),
                        name.clone(),
                        ns!(),
                        None,
                        |attr| *attr.name() == name,
                        CanGc::from_cx(cx),
                    );
                    Ok(true)
                },
                // Step 4.2.
                Some(false) => Ok(false),
            },
            Some(_index) => match force {
                // Step 5.
                None | Some(false) => {
                    self.remove_attribute_by_name(&name, CanGc::from_cx(cx));
                    Ok(false)
                },
                // Step 6.
                Some(true) => Ok(true),
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-element-setattribute>
    fn SetAttribute(
        &self,
        cx: &mut js::context::JSContext,
        name: DOMString,
        value: TrustedTypeOrString,
    ) -> ErrorResult {
        // Step 1. If qualifiedName does not match the Name production in XML,
        // then throw an "InvalidCharacterError" DOMException.
        if !is_valid_attribute_local_name(&name.str()) {
            return Err(Error::InvalidCharacter(None));
        }

        // Step 2. If this is in the HTML namespace and its node document is an HTML document,
        // then set qualifiedName to qualifiedName in ASCII lowercase.
        let name = self.parsed_name(name);

        // Step 3. Let verifiedValue be the result of calling get
        // Trusted Types-compliant attribute value with qualifiedName, null,
        // this, and value. [TRUSTED-TYPES]
        let value = TrustedTypePolicyFactory::get_trusted_types_compliant_attribute_value(
            cx,
            self.namespace(),
            self.local_name(),
            &name,
            None,
            value,
            &self.owner_global(),
        )?;

        // Step 4. Let attribute be the first attribute in this’s attribute list whose qualified name is qualifiedName, and null otherwise.
        // Step 5. If attribute is null, create an attribute whose local name is qualifiedName, value is verifiedValue, and node document
        // is this’s node document, then append this attribute to this, and then return.
        // Step 6. Change attribute to verifiedValue.
        let value = self.parse_attribute(&ns!(), &name, value);
        self.set_first_matching_attribute(
            name.clone(),
            value,
            name.clone(),
            ns!(),
            None,
            |attr| *attr.name() == name,
            CanGc::from_cx(cx),
        );
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-element-setattributens>
    fn SetAttributeNS(
        &self,
        cx: &mut js::context::JSContext,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        value: TrustedTypeOrString,
    ) -> ErrorResult {
        // Step 1. Let namespace, prefix, and localName be the result of passing namespace and qualifiedName to validate and extract.
        let (namespace, prefix, local_name) =
            domname::validate_and_extract(namespace, &qualified_name, domname::Context::Element)?;
        // Step 2. Let verifiedValue be the result of calling get
        // Trusted Types-compliant attribute value with localName, namespace, element, and value. [TRUSTED-TYPES]
        let value = TrustedTypePolicyFactory::get_trusted_types_compliant_attribute_value(
            cx,
            self.namespace(),
            self.local_name(),
            &local_name,
            Some(&namespace),
            value,
            &self.owner_global(),
        )?;
        // Step 3. Set an attribute value for this using localName, verifiedValue, and also prefix and namespace.
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.set_attribute_with_namespace(
            cx,
            local_name,
            value,
            LocalName::from(qualified_name),
            namespace,
            prefix,
        );
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-element-setattributenode>
    fn SetAttributeNode(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<Option<DomRoot<Attr>>> {
        self.set_attribute_node(cx, attr)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-setattributenodens>
    fn SetAttributeNodeNS(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<Option<DomRoot<Attr>>> {
        self.set_attribute_node(cx, attr)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-removeattribute>
    fn RemoveAttribute(&self, name: DOMString, can_gc: CanGc) {
        let name = self.parsed_name(name);
        self.remove_attribute_by_name(&name, can_gc);
    }

    /// <https://dom.spec.whatwg.org/#dom-element-removeattributens>
    fn RemoveAttributeNS(
        &self,
        cx: &mut js::context::JSContext,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) {
        let namespace = namespace_from_domstring(namespace);
        let local_name = LocalName::from(local_name);
        self.remove_attribute(&namespace, &local_name, CanGc::from_cx(cx));
    }

    /// <https://dom.spec.whatwg.org/#dom-element-removeattributenode>
    fn RemoveAttributeNode(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
    ) -> Fallible<DomRoot<Attr>> {
        self.remove_first_matching_attribute(|a| a == attr, CanGc::from_cx(cx))
            .ok_or(Error::NotFound(None))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-hasattribute>
    fn HasAttribute(&self, name: DOMString) -> bool {
        self.GetAttribute(name).is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-element-hasattributens>
    fn HasAttributeNS(&self, namespace: Option<DOMString>, local_name: DOMString) -> bool {
        self.GetAttributeNS(namespace, local_name).is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getelementsbytagname>
    fn GetElementsByTagName(&self, localname: DOMString, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_qualified_name(
            &window,
            self.upcast(),
            LocalName::from(localname),
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens>
    fn GetElementsByTagNameNS(
        &self,
        maybe_ns: Option<DOMString>,
        localname: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_tag_name_ns(&window, self.upcast(), localname, maybe_ns, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname>
    fn GetElementsByClassName(&self, classes: DOMString, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_class_name(&window, self.upcast(), classes, can_gc)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-getclientrects>
    fn GetClientRects(&self, can_gc: CanGc) -> DomRoot<DOMRectList> {
        let win = self.owner_window();
        let raw_rects = self.upcast::<Node>().border_boxes();
        let rects: Vec<DomRoot<DOMRect>> = raw_rects
            .map(|rect| {
                DOMRect::new(
                    win.upcast(),
                    rect.origin.x.to_f64_px(),
                    rect.origin.y.to_f64_px(),
                    rect.size.width.to_f64_px(),
                    rect.size.height.to_f64_px(),
                    can_gc,
                )
            })
            .collect();
        DOMRectList::new(&win, rects, can_gc)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect>
    fn GetBoundingClientRect(&self, can_gc: CanGc) -> DomRoot<DOMRect> {
        let win = self.owner_window();
        let rect = self.upcast::<Node>().border_box().unwrap_or_default();
        debug_assert!(rect.size.width.to_f64_px() >= 0.0 && rect.size.height.to_f64_px() >= 0.0);
        DOMRect::new(
            win.upcast(),
            rect.origin.x.to_f64_px(),
            rect.origin.y.to_f64_px(),
            rect.size.width.to_f64_px(),
            rect.size.height.to_f64_px(),
            can_gc,
        )
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scroll>
    fn Scroll(&self, options: &ScrollToOptions) {
        // Step 1
        let left = options.left.unwrap_or(self.ScrollLeft());
        let top = options.top.unwrap_or(self.ScrollTop());
        self.scroll(left, top, options.parent.behavior);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scroll>
    fn Scroll_(&self, x: f64, y: f64) {
        self.scroll(x, y, ScrollBehavior::Auto);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollto>
    fn ScrollTo(&self, options: &ScrollToOptions) {
        self.Scroll(options);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollto>
    fn ScrollTo_(&self, x: f64, y: f64) {
        self.Scroll_(x, y);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollby>
    fn ScrollBy(&self, options: &ScrollToOptions) {
        // Step 2
        let delta_left = options.left.unwrap_or(0.0f64);
        let delta_top = options.top.unwrap_or(0.0f64);
        let left = self.ScrollLeft();
        let top = self.ScrollTop();
        self.scroll(left + delta_left, top + delta_top, options.parent.behavior);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollby>
    fn ScrollBy_(&self, x: f64, y: f64) {
        let left = self.ScrollLeft();
        let top = self.ScrollTop();
        self.scroll(left + x, top + y, ScrollBehavior::Auto);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrolltop>
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
        if self.is_document_element() {
            if doc.quirks_mode() == QuirksMode::Quirks {
                return 0.0;
            }

            // Step 6
            return win.ScrollY() as f64;
        }

        // Step 7
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body()
        {
            return win.ScrollY() as f64;
        }

        // Step 8
        if !self.has_css_layout_box() {
            return 0.0;
        }

        // Step 9
        let point = win.scroll_offset_query(node);
        point.y.abs() as f64
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    // TODO(stevennovaryo): Need to update the scroll API to follow the spec since it is quite outdated.
    fn SetScrollTop(&self, y_: f64) {
        let behavior = ScrollBehavior::Auto;

        // Step 1, 2
        let y = if y_.is_finite() { y_ } else { 0.0 } as f32;

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
        if self.is_document_element() {
            if doc.quirks_mode() != QuirksMode::Quirks {
                win.scroll(win.ScrollX() as f32, y, behavior);
            }

            return;
        }

        // Step 9
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body()
        {
            win.scroll(win.ScrollX() as f32, y, behavior);
            return;
        }

        // Step 10
        if !self.has_scrolling_box() {
            return;
        }

        // Step 11
        win.scroll_an_element(self, self.ScrollLeft() as f32, y, behavior);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollleft>
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
        if self.is_document_element() {
            if doc.quirks_mode() != QuirksMode::Quirks {
                // Step 6
                return win.ScrollX() as f64;
            }

            return 0.0;
        }

        // Step 7
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body()
        {
            return win.ScrollX() as f64;
        }

        // Step 8
        if !self.has_css_layout_box() {
            return 0.0;
        }

        // Step 9
        let point = win.scroll_offset_query(node);
        point.x.abs() as f64
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollleft>
    fn SetScrollLeft(&self, x: f64) {
        let behavior = ScrollBehavior::Auto;

        // Step 1, 2
        let x = if x.is_finite() { x } else { 0.0 } as f32;

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
        if self.is_document_element() {
            if doc.quirks_mode() == QuirksMode::Quirks {
                return;
            }

            win.scroll(x, win.ScrollY() as f32, behavior);
            return;
        }

        // Step 9
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body()
        {
            win.scroll(x, win.ScrollY() as f32, behavior);
            return;
        }

        // Step 10
        if !self.has_scrolling_box() {
            return;
        }

        // Step 11
        win.scroll_an_element(self, x, self.ScrollTop() as f32, behavior);
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollintoview>
    fn ScrollIntoView(&self, arg: BooleanOrScrollIntoViewOptions) {
        let (behavior, block, inline, container) = match arg {
            // If arg is true:
            BooleanOrScrollIntoViewOptions::Boolean(true) => (
                ScrollBehavior::Auto,           // Step 1: Let behavior be "auto".
                ScrollLogicalPosition::Start,   // Step 2: Let block be "start".
                ScrollLogicalPosition::Nearest, // Step 3: Let inline be "nearest".
                None,                           // Step 4: Let container be null.
            ),
            // Step 5: If arg is a ScrollIntoViewOptions dictionary, set its properties
            // to the corresponding values in the dictionary.
            BooleanOrScrollIntoViewOptions::ScrollIntoViewOptions(options) => (
                options.parent.behavior,
                options.block,
                options.inline,
                // Step 5.4: If the container dictionary member of options is "nearest",
                // set container to the element.
                if options.container == ScrollIntoViewContainer::Nearest {
                    Some(self)
                } else {
                    None
                },
            ),
            // Step 6: Otherwise, if arg is false, then set block to "end".
            BooleanOrScrollIntoViewOptions::Boolean(false) => (
                ScrollBehavior::Auto,
                ScrollLogicalPosition::End,
                ScrollLogicalPosition::Nearest,
                None,
            ),
        };

        // Step 7: If the element does not have any associated box, or is not
        //         available to user-agent features, then return.
        if !self.has_css_layout_box() {
            return;
        }

        // Step 8: Scroll the element into view with behavior, block, inline, and container.
        self.scroll_into_view_with_options(
            behavior,
            ScrollAxisState::new_always_scroll_position(block),
            ScrollAxisState::new_always_scroll_position(inline),
            container,
            None,
        );

        // Step 9: Optionally perform some other action that brings the
        // element to the user’s attention.
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth>
    fn ScrollWidth(&self) -> i32 {
        self.upcast::<Node>().scroll_area().size.width
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollheight>
    fn ScrollHeight(&self) -> i32 {
        self.upcast::<Node>().scroll_area().size.height
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-clienttop>
    fn ClientTop(&self) -> i32 {
        self.client_rect().origin.y
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-clientleft>
    fn ClientLeft(&self) -> i32 {
        self.client_rect().origin.x
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-clientwidth>
    fn ClientWidth(&self) -> i32 {
        self.client_rect().size.width
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-clientheight>
    fn ClientHeight(&self) -> i32 {
        self.client_rect().size.height
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-currentcsszoom
    fn CurrentCSSZoom(&self) -> Finite<f64> {
        let window = self.owner_window();
        Finite::wrap(window.current_css_zoom_query(self.upcast::<Node>()) as f64)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-sethtmlunsafe>
    fn SetHTMLUnsafe(
        &self,
        cx: &mut js::context::JSContext,
        html: TrustedHTMLOrString,
    ) -> ErrorResult {
        // Step 1. Let compliantHTML be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, html, "Element setHTMLUnsafe", and "script".
        let html = TrustedHTML::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            html,
            "Element setHTMLUnsafe",
        )?;
        // Step 2. Let target be this's template contents if this is a template element; otherwise this.
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            DomRoot::upcast(template.Content(cx))
        } else {
            DomRoot::from_ref(self.upcast())
        };

        // Step 3. Unsafely set HTML given target, this, and compliantHTML
        Node::unsafely_set_html(&target, self, html, cx);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-gethtml>
    fn GetHTML(&self, cx: &mut js::context::JSContext, options: &GetHTMLOptions) -> DOMString {
        // > Element's getHTML(options) method steps are to return the result of HTML fragment serialization
        // > algorithm with this, options["serializableShadowRoots"], and options["shadowRoots"].
        self.upcast::<Node>().html_serialize(
            cx,
            TraversalScope::ChildrenOnly(None),
            options.serializableShadowRoots,
            options.shadowRoots.clone(),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn GetInnerHTML(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Fallible<TrustedHTMLOrNullIsEmptyString> {
        let qname = QualName::new(
            self.prefix().clone(),
            self.namespace().clone(),
            self.local_name().clone(),
        );

        // FIXME: This should use the fragment serialization algorithm, which takes
        // care of distinguishing between html/xml documents
        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>()
                .html_serialize(cx, ChildrenOnly(Some(qname)), false, vec![])
        } else {
            self.upcast::<Node>()
                .xml_serialize(XmlChildrenOnly(Some(qname)))?
        };

        Ok(TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(result))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn SetInnerHTML(
        &self,
        cx: &mut js::context::JSContext,
        value: TrustedHTMLOrNullIsEmptyString,
    ) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, the given value, "Element innerHTML", and "script".
        let value = TrustedHTML::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            value.convert(),
            "Element innerHTML",
        )?;
        // https://github.com/w3c/DOM-Parsing/issues/1
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            // Step 4: If context is a template element, then set context to
            // the template element's template contents (a DocumentFragment).
            DomRoot::upcast(template.Content(cx))
        } else {
            // Step 2: Let context be this.
            DomRoot::from_ref(self.upcast())
        };

        // Fast path for when the value is small, doesn't contain any markup and doesn't require
        // extra work to set innerHTML.
        if !self.node.has_weird_parser_insertion_mode() &&
            value.len() < 100 &&
            !value
                .as_bytes()
                .iter()
                .any(|c| matches!(*c, b'&' | b'\0' | b'<' | b'\r'))
        {
            return Node::SetTextContent(&target, cx, Some(value));
        }

        // Step 3: Let fragment be the result of invoking the fragment parsing algorithm steps
        // with context and compliantString.
        let frag = self.parse_fragment(value, cx)?;

        // Step 5: Replace all with fragment within context.
        Node::replace_all(cx, Some(frag.upcast()), &target);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn GetOuterHTML(
        &self,
        cx: &mut js::context::JSContext,
    ) -> Fallible<TrustedHTMLOrNullIsEmptyString> {
        // FIXME: This should use the fragment serialization algorithm, which takes
        // care of distinguishing between html/xml documents
        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>()
                .html_serialize(cx, IncludeNode, false, vec![])
        } else {
            self.upcast::<Node>().xml_serialize(XmlIncludeNode)?
        };

        Ok(TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(result))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn SetOuterHTML(
        &self,
        cx: &mut js::context::JSContext,
        value: TrustedHTMLOrNullIsEmptyString,
    ) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, the given value, "Element outerHTML", and "script".
        let value = TrustedHTML::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            value.convert(),
            "Element outerHTML",
        )?;
        let context_document = self.owner_document();
        let context_node = self.upcast::<Node>();
        // Step 2: Let parent be this's parent.
        let context_parent = match context_node.GetParentNode() {
            None => {
                // Step 3: If parent is null, return. There would be no way to
                // obtain a reference to the nodes created even if the remaining steps were run.
                return Ok(());
            },
            Some(parent) => parent,
        };

        let parent = match context_parent.type_id() {
            // Step 4: If parent is a Document, throw a "NoModificationAllowedError" DOMException.
            NodeTypeId::Document(_) => return Err(Error::NoModificationAllowed(None)),

            // Step 5: If parent is a DocumentFragment, set parent to the result of
            // creating an element given this's node document, "body", and the HTML namespace.
            NodeTypeId::DocumentFragment(_) => {
                let body_elem = Element::create(
                    cx,
                    QualName::new(None, ns!(html), local_name!("body")),
                    None,
                    &context_document,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Synchronous,
                    None,
                );
                DomRoot::upcast(body_elem)
            },
            _ => context_node.GetParentElement().unwrap(),
        };

        // Step 6: Let fragment be the result of invoking the
        // fragment parsing algorithm steps given parent and compliantString.
        let frag = parent.parse_fragment(value, cx)?;
        // Step 7: Replace this with fragment within this's parent.
        context_parent.ReplaceChild(cx, frag.upcast(), context_node)?;
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling>
    fn GetPreviousElementSibling(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .preceding_siblings()
            .find_map(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling>
    fn GetNextElementSibling(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .following_siblings()
            .find_map(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-children>
    fn Children(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::children(&window, self.upcast(), can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild>
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild>
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .find_map(DomRoot::downcast::<Element>)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-childelementcount>
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-prepend>
    fn Prepend(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().prepend(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-append>
    fn Append(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().append(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-replacechildren>
    fn ReplaceChildren(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_children(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-movebefore>
    fn MoveBefore(&self, cx: &mut JSContext, node: &Node, child: Option<&Node>) -> ErrorResult {
        self.upcast::<Node>().move_before(cx, node, child)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-before>
    fn Before(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().before(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-after>
    fn After(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().after(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-replacewith>
    fn ReplaceWith(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        self.upcast::<Node>().replace_with(cx, nodes)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-remove>
    fn Remove(&self, cx: &mut JSContext) {
        self.upcast::<Node>().remove_self(cx);
    }

    /// <https://dom.spec.whatwg.org/#dom-element-matches>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn Matches(&self, selectors: DOMString) -> Fallible<bool> {
        let document = self.owner_document();
        let url = document.url();
        let selectors = match SelectorParser::parse_author_origin_no_namespace(
            &selectors.str(),
            &UrlExtraData(url.get_arc()),
        ) {
            Err(_) => return Err(Error::Syntax(None)),
            Ok(selectors) => selectors,
        };

        // SAFETY: traced_self is unrooted, but we have a reference to "self" so it won't be freed.
        let traced_self = Dom::from_ref(self);
        let quirks_mode = document.quirks_mode();
        Ok(with_layout_state(|| {
            #[expect(unsafe_code)]
            let layout_element = unsafe { traced_self.to_layout() };
            dom_apis::element_matches(
                &ServoDangerousStyleElement::from(layout_element.upcast()),
                &selectors,
                quirks_mode,
            )
        }))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-webkitmatchesselector>
    fn WebkitMatchesSelector(&self, selectors: DOMString) -> Fallible<bool> {
        self.Matches(selectors)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-closest>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn Closest(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let document = self.owner_document();
        let url = document.url();
        let selectors = match SelectorParser::parse_author_origin_no_namespace(
            &selectors.str(),
            &UrlExtraData(url.get_arc()),
        ) {
            Err(_) => return Err(Error::Syntax(None)),
            Ok(selectors) => selectors,
        };

        // SAFETY: traced_self is unrooted, but we have a reference to "self" so it won't be freed.
        let traced_self = Dom::from_ref(self);
        let quirks_mode = document.quirks_mode();
        let closest_element = with_layout_state(|| {
            #[expect(unsafe_code)]
            let layout_element = unsafe { traced_self.to_layout() };
            dom_apis::element_closest(
                ServoDangerousStyleElement::from(layout_element.upcast()),
                &selectors,
                quirks_mode,
            )
        });
        Ok(closest_element.map(ServoDangerousStyleElement::rooted))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-insertadjacentelement>
    fn InsertAdjacentElement(
        &self,
        cx: &mut JSContext,
        where_: DOMString,
        element: &Element,
    ) -> Fallible<Option<DomRoot<Element>>> {
        let where_ = where_.parse::<AdjacentPosition>()?;
        let inserted_node = self.insert_adjacent(cx, where_, element.upcast())?;
        Ok(inserted_node.map(|node| DomRoot::downcast(node).unwrap()))
    }

    /// <https://dom.spec.whatwg.org/#dom-element-insertadjacenttext>
    fn InsertAdjacentText(
        &self,
        cx: &mut JSContext,
        where_: DOMString,
        data: DOMString,
    ) -> ErrorResult {
        // Step 1.
        let text = Text::new(cx, data, &self.owner_document());

        // Step 2.
        let where_ = where_.parse::<AdjacentPosition>()?;
        self.insert_adjacent(cx, where_, text.upcast()).map(|_| ())
    }

    /// <https://w3c.github.io/DOM-Parsing/#dom-element-insertadjacenthtml>
    fn InsertAdjacentHTML(
        &self,
        cx: &mut JSContext,
        position: DOMString,
        text: TrustedHTMLOrString,
    ) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, string, "Element insertAdjacentHTML", and "script".
        let text = TrustedHTML::get_trusted_type_compliant_string(
            cx,
            &self.owner_global(),
            text,
            "Element insertAdjacentHTML",
        )?;
        let position = position.parse::<AdjacentPosition>()?;

        // Step 2: Let context be null.
        // Step 3: Use the first matching item from this list:
        let context = match position {
            // If position is an ASCII case-insensitive match for the string "beforebegin"
            // If position is an ASCII case-insensitive match for the string "afterend"
            AdjacentPosition::BeforeBegin | AdjacentPosition::AfterEnd => {
                match self.upcast::<Node>().GetParentNode() {
                    // Step 3.2: If context is null or a Document, throw a "NoModificationAllowedError" DOMException.
                    Some(ref node) if node.is::<Document>() => {
                        return Err(Error::NoModificationAllowed(None));
                    },
                    None => return Err(Error::NoModificationAllowed(None)),
                    // Step 3.1: Set context to this's parent.
                    Some(node) => node,
                }
            },
            // If position is an ASCII case-insensitive match for the string "afterbegin"
            // If position is an ASCII case-insensitive match for the string "beforeend"
            AdjacentPosition::AfterBegin | AdjacentPosition::BeforeEnd => {
                // Set context to this.
                DomRoot::from_ref(self.upcast::<Node>())
            },
        };

        // Step 4.
        let context = Element::fragment_parsing_context(
            cx,
            &context.owner_doc(),
            context.downcast::<Element>(),
        );

        // Step 5: Let fragment be the result of invoking the
        // fragment parsing algorithm steps with context and compliantString.
        let fragment = context.parse_fragment(text, cx)?;

        // Step 6.
        self.insert_adjacent(cx, position, fragment.upcast())
            .map(|_| ())
    }

    // check-tidy: no specs after this line
    fn EnterFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.enter_formal_activation_state();
                Ok(())
            },
            None => Err(Error::NotSupported(None)),
        }
    }

    fn ExitFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.exit_formal_activation_state();
                Ok(())
            },
            None => Err(Error::NotSupported(None)),
        }
    }

    /// <https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen>
    fn RequestFullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        let doc = self.owner_document();
        doc.enter_fullscreen(self, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-attachshadow>
    fn AttachShadow(
        &self,
        cx: &mut JSContext,
        init: &ShadowRootInit,
    ) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1. Run attach a shadow root with this, init["mode"], init["clonable"], init["serializable"],
        // init["delegatesFocus"], and init["slotAssignment"].
        let shadow_root = self.attach_shadow(
            cx,
            IsUserAgentWidget::No,
            init.mode,
            init.clonable,
            init.serializable,
            init.delegatesFocus,
            init.slotAssignment,
        )?;

        // Step 2. Return this’s shadow root.
        Ok(shadow_root)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-shadowroot>
    fn GetShadowRoot(&self) -> Option<DomRoot<ShadowRoot>> {
        // Step 1. Let shadow be this’s shadow root.
        let shadow_or_none = self.shadow_root();

        // Step 2. If shadow is null or its mode is "closed", then return null.
        let shadow = shadow_or_none?;
        if shadow.Mode() == ShadowRootMode::Closed {
            return None;
        }

        // Step 3. Return shadow.
        Some(shadow)
    }

    /// <https://dom.spec.whatwg.org/#dom-element-customelementregistry>
    fn GetCustomElementRegistry(&self) -> Option<DomRoot<CustomElementRegistry>> {
        // The customElementRegistry getter steps are to return this’s custom element registry.
        self.custom_element_registry()
    }

    /// <https://w3c.github.io/aria/#ref-for-dom-ariamixin-role-1>
    fn GetRole(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("role"))
    }

    /// <https://w3c.github.io/aria/#ref-for-dom-ariamixin-role-1>
    fn SetRole(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("role"), value);
    }

    fn GetAriaAtomic(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-atomic"))
    }

    fn SetAriaAtomic(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-atomic"), value);
    }

    fn GetAriaAutoComplete(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-autocomplete"))
    }

    fn SetAriaAutoComplete(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-autocomplete"), value);
    }

    fn GetAriaBrailleLabel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-braillelabel"))
    }

    fn SetAriaBrailleLabel(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-braillelabel"), value);
    }

    fn GetAriaBrailleRoleDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-brailleroledescription"))
    }

    fn SetAriaBrailleRoleDescription(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-brailleroledescription"), value);
    }

    fn GetAriaBusy(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-busy"))
    }

    fn SetAriaBusy(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-busy"), value);
    }

    fn GetAriaChecked(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-checked"))
    }

    fn SetAriaChecked(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-checked"), value);
    }

    fn GetAriaColCount(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colcount"))
    }

    fn SetAriaColCount(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-colcount"), value);
    }

    fn GetAriaColIndex(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colindex"))
    }

    fn SetAriaColIndex(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-colindex"), value);
    }

    fn GetAriaColIndexText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colindextext"))
    }

    fn SetAriaColIndexText(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-colindextext"), value);
    }

    fn GetAriaColSpan(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colspan"))
    }

    fn SetAriaColSpan(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-colspan"), value);
    }

    fn GetAriaCurrent(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-current"))
    }

    fn SetAriaCurrent(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-current"), value);
    }

    fn GetAriaDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-description"))
    }

    fn SetAriaDescription(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-description"), value);
    }

    fn GetAriaDisabled(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-disabled"))
    }

    fn SetAriaDisabled(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-disabled"), value);
    }

    fn GetAriaExpanded(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-expanded"))
    }

    fn SetAriaExpanded(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-expanded"), value);
    }

    fn GetAriaHasPopup(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-haspopup"))
    }

    fn SetAriaHasPopup(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-haspopup"), value);
    }

    fn GetAriaHidden(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-hidden"))
    }

    fn SetAriaHidden(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-hidden"), value);
    }

    fn GetAriaInvalid(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-invalid"))
    }

    fn SetAriaInvalid(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-invalid"), value);
    }

    fn GetAriaKeyShortcuts(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-keyshortcuts"))
    }

    fn SetAriaKeyShortcuts(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-keyshortcuts"), value);
    }

    fn GetAriaLabel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-label"))
    }

    fn SetAriaLabel(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-label"), value);
    }

    fn GetAriaLevel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-level"))
    }

    fn SetAriaLevel(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-level"), value);
    }

    fn GetAriaLive(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-live"))
    }

    fn SetAriaLive(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-live"), value);
    }

    fn GetAriaModal(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-modal"))
    }

    fn SetAriaModal(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-modal"), value);
    }

    fn GetAriaMultiLine(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-multiline"))
    }

    fn SetAriaMultiLine(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-multiline"), value);
    }

    fn GetAriaMultiSelectable(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-multiselectable"))
    }

    fn SetAriaMultiSelectable(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-multiselectable"), value);
    }

    fn GetAriaOrientation(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-orientation"))
    }

    fn SetAriaOrientation(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-orientation"), value);
    }

    fn GetAriaPlaceholder(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-placeholder"))
    }

    fn SetAriaPlaceholder(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-placeholder"), value);
    }

    fn GetAriaPosInSet(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-posinset"))
    }

    fn SetAriaPosInSet(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-posinset"), value);
    }

    fn GetAriaPressed(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-pressed"))
    }

    fn SetAriaPressed(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-pressed"), value);
    }

    fn GetAriaReadOnly(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-readonly"))
    }

    fn SetAriaReadOnly(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-readonly"), value);
    }

    fn GetAriaRelevant(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-relevant"))
    }

    fn SetAriaRelevant(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-relevant"), value);
    }

    fn GetAriaRequired(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-required"))
    }

    fn SetAriaRequired(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-required"), value);
    }

    fn GetAriaRoleDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-roledescription"))
    }

    fn SetAriaRoleDescription(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-roledescription"), value);
    }

    fn GetAriaRowCount(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowcount"))
    }

    fn SetAriaRowCount(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-rowcount"), value);
    }

    fn GetAriaRowIndex(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowindex"))
    }

    fn SetAriaRowIndex(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-rowindex"), value);
    }

    fn GetAriaRowIndexText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowindextext"))
    }

    fn SetAriaRowIndexText(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-rowindextext"), value);
    }

    fn GetAriaRowSpan(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowspan"))
    }

    fn SetAriaRowSpan(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-rowspan"), value);
    }

    fn GetAriaSelected(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-selected"))
    }

    fn SetAriaSelected(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-selected"), value);
    }

    fn GetAriaSetSize(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-setsize"))
    }

    fn SetAriaSetSize(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-setsize"), value);
    }

    fn GetAriaSort(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-sort"))
    }

    fn SetAriaSort(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-sort"), value);
    }

    fn GetAriaValueMax(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuemax"))
    }

    fn SetAriaValueMax(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-valuemax"), value);
    }

    fn GetAriaValueMin(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuemin"))
    }

    fn SetAriaValueMin(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-valuemin"), value);
    }

    fn GetAriaValueNow(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuenow"))
    }

    fn SetAriaValueNow(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-valuenow"), value);
    }

    fn GetAriaValueText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuetext"))
    }

    fn SetAriaValueText(&self, cx: &mut JSContext, value: Option<DOMString>) {
        self.set_nullable_string_attribute(cx, &local_name!("aria-valuetext"), value);
    }

    /// <https://dom.spec.whatwg.org/#dom-slotable-assignedslot>
    fn GetAssignedSlot(&self) -> Option<DomRoot<HTMLSlotElement>> {
        let cx = GlobalScope::get_cx();

        // > The assignedSlot getter steps are to return the result of
        // > find a slot given this and with the open flag set.
        rooted!(in(*cx) let slottable = Slottable(Dom::from_ref(self.upcast::<Node>())));
        slottable.find_a_slot(true)
    }

    /// <https://drafts.csswg.org/css-shadow-parts/#dom-element-part>
    fn Part(&self) -> DomRoot<DOMTokenList> {
        self.ensure_rare_data().part.or_init(|| {
            DOMTokenList::new(self, &local_name!("part"), None, CanGc::deprecated_note())
        })
    }
}

impl VirtualMethods for Element {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Node>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        // FIXME: This should be more fine-grained, not all elements care about these.
        if attr.local_name() == &local_name!("lang") {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        let node = self.upcast::<Node>();
        let doc = node.owner_doc();
        match *attr.local_name() {
            local_name!("style") => self.update_style_attribute(attr, mutation),
            local_name!("id") => {
                // https://dom.spec.whatwg.org/#ref-for-concept-element-attributes-change-ext%E2%91%A2
                *self.id_attribute.borrow_mut() = mutation.new_value(attr).and_then(|value| {
                    let value = value.as_atom();
                    if value != &atom!("") {
                        // Step 2. Otherwise, if localName is id, namespace is null, then set element’s ID to value.
                        Some(value.clone())
                    } else {
                        // Step 1. If localName is id, namespace is null, and value is null or the empty string, then unset element’s ID.
                        None
                    }
                });

                let containing_shadow_root = self.containing_shadow_root();
                if node.is_in_a_document_tree() || node.is_in_a_shadow_tree() {
                    let value = attr.value().as_atom().clone();
                    match mutation {
                        AttributeMutation::Set(old_value, _) => {
                            if let Some(old_value) = old_value {
                                let old_value = old_value.as_atom().clone();
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.unregister_element_id(
                                        self,
                                        old_value,
                                        CanGc::from_cx(cx),
                                    );
                                } else {
                                    doc.unregister_element_id(self, old_value, CanGc::from_cx(cx));
                                }
                            }
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.register_element_id(
                                        self,
                                        value,
                                        CanGc::from_cx(cx),
                                    );
                                } else {
                                    doc.register_element_id(self, value, CanGc::from_cx(cx));
                                }
                            }
                        },
                        AttributeMutation::Removed => {
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.unregister_element_id(
                                        self,
                                        value,
                                        CanGc::from_cx(cx),
                                    );
                                } else {
                                    doc.unregister_element_id(self, value, CanGc::from_cx(cx));
                                }
                            }
                        },
                    }
                }
            },
            local_name!("name") => {
                // Keep the name in rare data for fast access
                self.ensure_rare_data().name_attribute =
                    mutation.new_value(attr).and_then(|value| {
                        let value = value.as_atom();
                        if value != &atom!("") {
                            Some(value.clone())
                        } else {
                            None
                        }
                    });
                // Keep the document name_map up to date
                // (if we're not in shadow DOM)
                if node.is_connected() && node.containing_shadow_root().is_none() {
                    let value = attr.value().as_atom().clone();
                    match mutation {
                        AttributeMutation::Set(old_value, _) => {
                            if let Some(old_value) = old_value {
                                let old_value = old_value.as_atom().clone();
                                doc.unregister_element_name(self, old_value);
                            }
                            if value != atom!("") {
                                doc.register_element_name(self, value);
                            }
                        },
                        AttributeMutation::Removed => {
                            if value != atom!("") {
                                doc.unregister_element_name(self, value);
                            }
                        },
                    }
                }
            },
            local_name!("slot") => {
                // Update slottable data
                let cx = GlobalScope::get_cx();

                rooted!(in(*cx) let slottable = Slottable(Dom::from_ref(self.upcast::<Node>())));

                // Slottable name change steps from https://dom.spec.whatwg.org/#light-tree-slotables
                if let Some(assigned_slot) = slottable.assigned_slot() {
                    assigned_slot.assign_slottables();
                }
                slottable.assign_a_slot();
            },
            _ => {
                // FIXME(emilio): This is pretty dubious, and should be done in
                // the relevant super-classes.
                if attr.namespace() == &ns!() && attr.local_name() == &local_name!("src") {
                    node.dirty(NodeDamage::Other);
                }
            },
        };

        // TODO: This should really only take into account the actual attributes that are used
        // for the content attribute property.
        if self
            .upcast::<Node>()
            .get_flag(NodeFlags::USES_ATTR_IN_CONTENT_ATTRIBUTE)
        {
            node.dirty(NodeDamage::ContentOrHeritage);
        }

        // Make sure we rev the version even if we didn't dirty the node. If we
        // don't do this, various attribute-dependent htmlcollections (like those
        // generated by getElementsByClassName) might become stale.
        node.rev_version();

        // Notify devtools that the DOM changed
        let window = self.owner_window();
        if window.live_devtools_updates() {
            let global = window.upcast::<GlobalScope>();
            if let Some(sender) = global.devtools_chan() {
                let pipeline_id = global.pipeline_id();
                if ScriptThread::devtools_want_updates_for_node(pipeline_id, self.upcast()) {
                    let devtools_message = ScriptToDevtoolsControlMsg::DomMutation(
                        pipeline_id,
                        DomMutation::AttributeModified {
                            node: self.upcast::<Node>().unique_id(pipeline_id),
                            attribute_name: attr.local_name().to_string(),
                            new_value: mutation.new_value(attr).map(|value| value.to_string()),
                        },
                    );
                    sender.send(devtools_message).unwrap();
                }
            }
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("id") => AttrValue::Atom(value.into()),
            local_name!("name") => AttrValue::Atom(value.into()),
            local_name!("class") | local_name!("part") => {
                AttrValue::from_serialized_tokenlist(value.into())
            },
            local_name!("exportparts") => AttrValue::from_shadow_parts(value.into()),
            local_name!("tabindex") => AttrValue::from_i32(value.into(), -1),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }

        if let Some(f) = self.as_maybe_form_control() {
            f.bind_form_control_to_tree(CanGc::from_cx(cx));
        }

        let doc = self.owner_document();

        if let Some(ref shadow_root) = self.shadow_root() {
            shadow_root.bind_to_tree(cx, context);
        }

        if !context.is_in_tree() {
            return;
        }

        if let Some(ref id) = *self.id_attribute.borrow() {
            if let Some(shadow_root) = self.containing_shadow_root() {
                shadow_root.register_element_id(self, id.clone(), CanGc::from_cx(cx));
            } else {
                doc.register_element_id(self, id.clone(), CanGc::from_cx(cx));
            }
        }
        if let Some(ref name) = self.name_attribute() {
            if self.containing_shadow_root().is_none() {
                doc.register_element_name(self, name.clone());
            }
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        if let Some(f) = self.as_maybe_form_control() {
            // TODO: The valid state of ancestors might be wrong if the form control element
            // has a fieldset ancestor, for instance: `<form><fieldset><input>`,
            // if `<input>` is unbound, `<form><fieldset>` should trigger a call to `update_validity()`.
            f.unbind_form_control_from_tree(can_gc);
        }

        if !context.tree_is_in_a_document_tree && !context.tree_is_in_a_shadow_tree {
            return;
        }

        let doc = self.owner_document();

        let fullscreen = doc.fullscreen_element();
        if fullscreen.as_deref() == Some(self) {
            doc.exit_fullscreen(can_gc);
        }
        if let Some(ref value) = *self.id_attribute.borrow() {
            if let Some(ref shadow_root) = self.containing_shadow_root() {
                // Only unregister the element id if the node was disconnected from it's shadow root
                // (as opposed to the whole shadow tree being disconnected as a whole)
                if !self.upcast::<Node>().is_in_a_shadow_tree() {
                    shadow_root.unregister_element_id(self, value.clone(), can_gc);
                }
            } else {
                doc.unregister_element_id(self, value.clone(), can_gc);
            }
        }
        if let Some(ref value) = self.name_attribute() {
            if self.containing_shadow_root().is_none() {
                doc.unregister_element_name(self, value.clone());
            }
        }
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(cx, mutation);
        }

        let flags = self.get_selector_flags();
        if flags.intersects(ElementSelectorFlags::HAS_SLOW_SELECTOR) {
            // All children of this node need to be restyled when any child changes.
            self.upcast::<Node>().dirty(NodeDamage::Other);
        } else {
            if flags.intersects(ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
                if let Some(next_child) = mutation.next_child() {
                    for child in next_child.inclusively_following_siblings_unrooted(cx.no_gc()) {
                        if child.is::<Element>() {
                            child.dirty(NodeDamage::Other);
                        }
                    }
                }
            }
            if flags.intersects(ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR) {
                if let Some(child) = mutation.modified_edge_element(cx.no_gc()) {
                    child.dirty(NodeDamage::Other);
                }
            }
        }
    }

    fn adopting_steps(&self, cx: &mut JSContext, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(cx, old_doc);

        if self.owner_document().is_html_document() != old_doc.is_html_document() {
            self.tag_name.clear();
        }
    }

    fn post_connection_steps(&self, cx: &mut js::context::JSContext) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps(cx);
        }

        self.update_nonce_post_connection();
    }

    /// <https://html.spec.whatwg.org/multipage/#nonce-attributes%3Aconcept-node-clone-ext>
    fn cloning_steps(
        &self,
        cx: &mut JSContext,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(cx, copy, maybe_doc, clone_children);
        }
        let elem = copy.downcast::<Element>().unwrap();
        if let Some(rare_data) = self.rare_data().as_ref() {
            elem.update_nonce_internal_slot(rare_data.cryptographic_nonce.clone());
        }
    }
}
impl Element {
    pub(crate) fn client_rect(&self) -> Rect<i32, CSSPixel> {
        let doc = self.node.owner_doc();

        if let Some(rect) = self
            .rare_data()
            .as_ref()
            .and_then(|data| data.client_rect.as_ref())
            .and_then(|rect| rect.get().ok())
        {
            if doc.restyle_reason().is_empty() {
                return rect;
            }
        }

        let mut rect = self.upcast::<Node>().client_rect();
        let in_quirks_mode = doc.quirks_mode() == QuirksMode::Quirks;

        if (in_quirks_mode && doc.GetBody().as_deref() == self.downcast::<HTMLElement>()) ||
            (!in_quirks_mode && self.is_document_element())
        {
            rect.size = doc.window().viewport_details().size.round().to_i32();
        }

        self.ensure_rare_data().client_rect = Some(self.owner_window().cache_layout_value(rect));
        rect
    }

    pub(crate) fn as_maybe_activatable(&self) -> Option<&dyn Activatable> {
        let element = match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement,
            )) => {
                let element = self.downcast::<HTMLInputElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement,
            )) => {
                let element = self.downcast::<HTMLButtonElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) => {
                let element = self.downcast::<HTMLAnchorElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLLabelElement,
            )) => {
                let element = self.downcast::<HTMLLabelElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) => {
                let element = self.downcast::<HTMLSelectElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLElement)) => {
                let element = self.downcast::<HTMLElement>().unwrap();
                Some(element as &dyn Activatable)
            },
            _ => None,
        };
        element.and_then(|elem| {
            if elem.is_instance_activatable() {
                Some(elem)
            } else {
                None
            }
        })
    }

    pub(crate) fn as_stylesheet_owner(&self) -> Option<&dyn StylesheetOwner> {
        if let Some(s) = self.downcast::<HTMLStyleElement>() {
            return Some(s as &dyn StylesheetOwner);
        }

        if let Some(l) = self.downcast::<HTMLLinkElement>() {
            return Some(l as &dyn StylesheetOwner);
        }

        None
    }

    // https://html.spec.whatwg.org/multipage/#category-submit
    pub(crate) fn as_maybe_validatable(&self) -> Option<&dyn Validatable> {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement,
            )) => {
                let element = self.downcast::<HTMLInputElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement,
            )) => {
                let element = self.downcast::<HTMLButtonElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLObjectElement,
            )) => {
                let element = self.downcast::<HTMLObjectElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) => {
                let element = self.downcast::<HTMLSelectElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLTextAreaElement,
            )) => {
                let element = self.downcast::<HTMLTextAreaElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLFieldSetElement,
            )) => {
                let element = self.downcast::<HTMLFieldSetElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLOutputElement,
            )) => {
                let element = self.downcast::<HTMLOutputElement>().unwrap();
                Some(element as &dyn Validatable)
            },
            _ => None,
        }
    }

    pub(crate) fn is_invalid(&self, needs_update: bool, can_gc: CanGc) -> bool {
        if let Some(validatable) = self.as_maybe_validatable() {
            if needs_update {
                validatable
                    .validity_state(can_gc)
                    .perform_validation_and_update(ValidationFlags::all(), can_gc);
            }
            return validatable.is_instance_validatable() &&
                !validatable.satisfies_constraints(can_gc);
        }

        if let Some(internals) = self.get_element_internals() {
            return internals.is_invalid(can_gc);
        }
        false
    }

    pub(crate) fn is_instance_validatable(&self) -> bool {
        if let Some(validatable) = self.as_maybe_validatable() {
            return validatable.is_instance_validatable();
        }
        if let Some(internals) = self.get_element_internals() {
            return internals.is_instance_validatable();
        }
        false
    }

    pub(crate) fn init_state_for_internals(&self) {
        self.set_enabled_state(true);
        self.set_state(ElementState::VALID, true);
        self.set_state(ElementState::INVALID, false);
    }

    pub(crate) fn click_in_progress(&self) -> bool {
        self.upcast::<Node>().get_flag(NodeFlags::CLICK_IN_PROGRESS)
    }

    pub(crate) fn set_click_in_progress(&self, click: bool) {
        self.upcast::<Node>()
            .set_flag(NodeFlags::CLICK_IN_PROGRESS, click)
    }

    pub fn state(&self) -> ElementState {
        self.state.get()
    }

    pub(crate) fn set_state(&self, which: ElementState, value: bool) {
        let mut state = self.state.get();
        let previous_state = state;
        if value {
            state.insert(which);
        } else {
            state.remove(which);
        }

        if previous_state == state {
            // Nothing to do
            return;
        }

        // Add a pending restyle for this node which captures a snapshot of the state
        // before the change.
        {
            let document = self.owner_document();
            let mut entry = document.ensure_pending_restyle(self);
            if entry.snapshot.is_none() {
                entry.snapshot = Some(Snapshot::new());
            }
            let snapshot = entry.snapshot.as_mut().unwrap();
            if snapshot.state.is_none() {
                snapshot.state = Some(self.state());
            }
        }

        self.state.set(state);
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-selector-active>
    pub(crate) fn set_active_state(&self, value: bool) {
        self.set_state(ElementState::ACTIVE, value);

        if let Some(parent) = self.upcast::<Node>().GetParentElement() {
            parent.set_active_state(value);
        }
    }

    pub(crate) fn focus_state(&self) -> bool {
        self.state.get().contains(ElementState::FOCUS)
    }

    pub(crate) fn set_focus_state(&self, value: bool) {
        self.set_state(ElementState::FOCUS, value);
    }

    pub(crate) fn set_hover_state(&self, value: bool) {
        self.set_state(ElementState::HOVER, value);
    }

    pub(crate) fn enabled_state(&self) -> bool {
        self.state.get().contains(ElementState::ENABLED)
    }

    pub(crate) fn set_enabled_state(&self, value: bool) {
        self.set_state(ElementState::ENABLED, value)
    }

    pub(crate) fn disabled_state(&self) -> bool {
        self.state.get().contains(ElementState::DISABLED)
    }

    pub(crate) fn set_disabled_state(&self, value: bool) {
        self.set_state(ElementState::DISABLED, value)
    }

    pub(crate) fn read_write_state(&self) -> bool {
        self.state.get().contains(ElementState::READWRITE)
    }

    pub(crate) fn set_read_write_state(&self, value: bool) {
        self.set_state(ElementState::READWRITE, value)
    }

    pub(crate) fn set_open_state(&self, value: bool) {
        self.set_state(ElementState::OPEN, value);
    }

    pub(crate) fn set_placeholder_shown_state(&self, value: bool) {
        self.set_state(ElementState::PLACEHOLDER_SHOWN, value);
    }

    pub(crate) fn set_modal_state(&self, value: bool) {
        self.set_state(ElementState::MODAL, value);
    }

    pub(crate) fn set_target_state(&self, value: bool) {
        self.set_state(ElementState::URLTARGET, value)
    }

    pub(crate) fn set_fullscreen_state(&self, value: bool) {
        self.set_state(ElementState::FULLSCREEN, value)
    }

    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) fn is_connected(&self) -> bool {
        self.upcast::<Node>().is_connected()
    }

    // https://html.spec.whatwg.org/multipage/#cannot-navigate
    pub(crate) fn cannot_navigate(&self) -> bool {
        let document = self.owner_document();

        // Step 1.
        !document.is_fully_active() ||
            (
                // Step 2.
                !self.is::<HTMLAnchorElement>() && !self.is_connected()
            )
    }
}

impl Element {
    pub(crate) fn check_ancestors_disabled_state_for_form_control(&self) {
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

    pub(crate) fn check_parent_disabled_state_for_option(&self) {
        if self.disabled_state() {
            return;
        }
        let node = self.upcast::<Node>();
        if let Some(ref parent) = node.GetParentNode() {
            if parent.is::<HTMLOptGroupElement>() &&
                parent.downcast::<Element>().unwrap().disabled_state()
            {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
            }
        }
    }

    pub(crate) fn check_disabled_attribute(&self) {
        let has_disabled_attrib = self.has_attribute(&local_name!("disabled"));
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }

    pub(crate) fn update_read_write_state_from_readonly_attribute(&self) {
        let has_readonly_attribute = self.has_attribute(&local_name!("readonly"));
        self.set_read_write_state(has_readonly_attribute);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum AttributeMutationReason {
    ByCloning,
    ByParser,
    Directly,
}

#[derive(Clone, Copy)]
pub(crate) enum AttributeMutation<'a> {
    /// The attribute is set, keep track of old value.
    /// <https://dom.spec.whatwg.org/#attribute-is-set>
    Set(Option<&'a AttrValue>, AttributeMutationReason),

    /// The attribute is removed.
    /// <https://dom.spec.whatwg.org/#attribute-is-removed>
    Removed,
}

impl AttributeMutation<'_> {
    pub(crate) fn is_removal(&self) -> bool {
        match *self {
            AttributeMutation::Removed => true,
            AttributeMutation::Set(..) => false,
        }
    }

    pub(crate) fn new_value<'b>(&self, attr: &'b Attr) -> Option<Ref<'b, AttrValue>> {
        match *self {
            AttributeMutation::Set(..) => Some(attr.value()),
            AttributeMutation::Removed => None,
        }
    }
}

/// A holder for an element's "tag name", which will be lazily
/// resolved and cached. Should be reset when the document
/// owner changes.
#[derive(JSTraceable, MallocSizeOf)]
struct TagName {
    #[no_trace]
    ptr: DomRefCell<Option<LocalName>>,
}

impl TagName {
    fn new() -> TagName {
        TagName {
            ptr: DomRefCell::new(None),
        }
    }

    /// Retrieve a copy of the current inner value. If it is `None`, it is
    /// initialized with the result of `cb` first.
    fn or_init<F>(&self, cb: F) -> LocalName
    where
        F: FnOnce() -> LocalName,
    {
        match &mut *self.ptr.borrow_mut() {
            &mut Some(ref name) => name.clone(),
            ptr => {
                let name = cb();
                *ptr = Some(name.clone());
                name
            },
        }
    }

    /// Clear the cached tag name, so that it will be re-calculated the
    /// next time that `or_init()` is called.
    fn clear(&self) {
        *self.ptr.borrow_mut() = None;
    }
}

/// <https://html.spec.whatwg.org/multipage/#cors-settings-attribute>
pub(crate) fn reflect_cross_origin_attribute(element: &Element) -> Option<DOMString> {
    element
        .get_attribute(&local_name!("crossorigin"))
        .map(|attribute| {
            let value = attribute.value().to_ascii_lowercase();
            if value == "anonymous" || value == "use-credentials" {
                DOMString::from(value)
            } else {
                DOMString::from("anonymous")
            }
        })
}

pub(crate) fn set_cross_origin_attribute(
    cx: &mut JSContext,
    element: &Element,
    value: Option<DOMString>,
) {
    match value {
        Some(val) => {
            element.set_string_attribute(&local_name!("crossorigin"), val, CanGc::from_cx(cx))
        },
        None => {
            element.remove_attribute(&ns!(), &local_name!("crossorigin"), CanGc::from_cx(cx));
        },
    }
}

/// <https://html.spec.whatwg.org/multipage/#referrer-policy-attribute>
pub(crate) fn reflect_referrer_policy_attribute(element: &Element) -> DOMString {
    element
        .get_attribute(&local_name!("referrerpolicy"))
        .map(|attribute| {
            let value = attribute.value().to_ascii_lowercase();
            if value == "no-referrer" ||
                value == "no-referrer-when-downgrade" ||
                value == "same-origin" ||
                value == "origin" ||
                value == "strict-origin" ||
                value == "origin-when-cross-origin" ||
                value == "strict-origin-when-cross-origin" ||
                value == "unsafe-url"
            {
                DOMString::from(value)
            } else {
                DOMString::new()
            }
        })
        .unwrap_or_default()
}

pub(crate) fn referrer_policy_for_element(element: &Element) -> ReferrerPolicy {
    element
        .get_attribute(&local_name!("referrerpolicy"))
        .map(|attribute| ReferrerPolicy::from(&**attribute.value()))
        .unwrap_or(element.owner_document().get_referrer_policy())
}

pub(crate) fn cors_setting_for_element(element: &Element) -> Option<CorsSettings> {
    element
        .get_attribute(&local_name!("crossorigin"))
        .map(|attribute| CorsSettings::from_enumerated_attribute(&attribute.value()))
}

/// <https://html.spec.whatwg.org/multipage/#cors-settings-attribute-credentials-mode>
pub(crate) fn cors_settings_attribute_credential_mode(element: &Element) -> CredentialsMode {
    element
        .get_attribute(&local_name!("crossorigin"))
        .map(|attr| {
            if attr.value().eq_ignore_ascii_case("use-credentials") {
                CredentialsMode::Include
            } else {
                // The attribute's invalid value default and empty value default are both the Anonymous state.
                CredentialsMode::CredentialsSameOrigin
            }
        })
        // The attribute's missing value default is the No CORS state, which defaults to "same-origin"
        .unwrap_or(CredentialsMode::CredentialsSameOrigin)
}

pub(crate) fn is_element_affected_by_legacy_background_presentational_hint(
    namespace: &Namespace,
    local_name: &LocalName,
) -> bool {
    *namespace == ns!(html) &&
        matches!(
            *local_name,
            local_name!("body") |
                local_name!("table") |
                local_name!("thead") |
                local_name!("tbody") |
                local_name!("tfoot") |
                local_name!("tr") |
                local_name!("td") |
                local_name!("th")
        )
}
