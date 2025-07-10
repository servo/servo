/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Element nodes.

use std::borrow::Cow;
use std::cell::{Cell, LazyCell};
use std::default::Default;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::{fmt, mem};

use cssparser::{Parser as CssParser, ParserInput as CssParserInput, match_ignore_ascii_case};
use devtools_traits::AttrInfo;
use dom_struct::dom_struct;
use embedder_traits::InputMethodType;
use euclid::default::{Rect, Size2D};
use html5ever::serialize::TraversalScope;
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::{LocalName, Namespace, Prefix, QualName, local_name, namespace_prefix, ns};
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::HandleObject;
use layout_api::LayoutDamage;
use net_traits::ReferrerPolicy;
use net_traits::request::CorsSettings;
use selectors::Element as SelectorsElement;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::{BLOOM_HASH_MASK, BloomFilter};
use selectors::matching::{ElementSelectorFlags, MatchingContext};
use selectors::sink::Push;
use servo_arc::Arc;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::media_queries::MediaList;
use style::parser::ParserContext as CssParserContext;
use style::properties::longhands::{
    self, background_image, border_spacing, font_family, font_size,
};
use style::properties::{
    ComputedValues, Importance, PropertyDeclaration, PropertyDeclarationBlock,
    parse_style_attribute,
};
use style::rule_tree::CascadeLevel;
use style::selector_parser::{
    NonTSPseudoClass, PseudoElement, RestyleDamage, SelectorImpl, SelectorParser,
    extended_filtering,
};
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::layer_rule::LayerOrder;
use style::stylesheets::{CssRuleType, Origin as CssOrigin, UrlExtraData};
use style::values::computed::Overflow;
use style::values::generics::NonNegative;
use style::values::generics::position::PreferredRatio;
use style::values::generics::ratio::Ratio;
use style::values::{AtomIdent, AtomString, CSSFloat, computed, specified};
use style::{ArcSlice, CaseSensitivityExt, dom_apis, thread_state};
use style_traits::ParsingMode as CssParsingMode;
use stylo_atoms::Atom;
use stylo_dom::ElementState;
use xml5ever::serialize::TraversalScope::{
    ChildrenOnly as XmlChildrenOnly, IncludeNode as XmlIncludeNode,
};

use crate::conversions::Convert;
use crate::dom::activation::Activatable;
use crate::dom::attr::{Attr, AttrHelpersForLayout, is_relevant_attribute};
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::{
    ElementMethods, GetHTMLOptions, ShadowRootInit,
};
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMethods, ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    ScrollBehavior, ScrollToOptions, WindowMethods,
};
use crate::dom::bindings::codegen::UnionTypes::{
    NodeOrString, TrustedHTMLOrNullIsEmptyString, TrustedHTMLOrString, TrustedScriptURLOrUSVString,
};
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom, ToLayout};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::xmlname::{
    matches_name_production, namespace_from_domstring, validate_and_extract,
};
use crate::dom::characterdata::CharacterData;
use crate::dom::create::create_element;
use crate::dom::csp::{CspReporting, InlineCheckType};
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReaction, CustomElementState,
    is_valid_custom_element_name,
};
use crate::dom::document::{Document, LayoutDocumentHelpers, determine_policy_for_token};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domrect::DOMRect;
use crate::dom::domrectlist::DOMRectList;
use crate::dom::domtokenlist::DOMTokenList;
use crate::dom::elementinternals::ElementInternals;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlanchorelement::HTMLAnchorElement;
use crate::dom::htmlbodyelement::{HTMLBodyElement, HTMLBodyElementLayoutHelpers};
use crate::dom::htmlbuttonelement::HTMLButtonElement;
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlfontelement::{HTMLFontElement, HTMLFontElementLayoutHelpers};
use crate::dom::htmlformelement::FormControlElementHelpers;
use crate::dom::htmlhrelement::{HTMLHRElement, HTMLHRLayoutHelpers, SizePresentationalHint};
use crate::dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use crate::dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use crate::dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlobjectelement::HTMLObjectElement;
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloutputelement::HTMLOutputElement;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementLayoutHelpers};
use crate::dom::htmltablecolelement::{HTMLTableColElement, HTMLTableColElementLayoutHelpers};
use crate::dom::htmltableelement::{HTMLTableElement, HTMLTableElementLayoutHelpers};
use crate::dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementLayoutHelpers};
use crate::dom::htmltablesectionelement::{
    HTMLTableSectionElement, HTMLTableSectionElementLayoutHelpers,
};
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use crate::dom::htmlvideoelement::{HTMLVideoElement, LayoutHTMLVideoElementHelpers};
use crate::dom::intersectionobserver::{IntersectionObserver, IntersectionObserverRegistration};
use crate::dom::mutationobserver::{Mutation, MutationObserver};
use crate::dom::namednodemap::NamedNodeMap;
use crate::dom::node::{
    BindContext, ChildrenMutation, CloneChildrenFlag, LayoutNodeHelpers, Node, NodeDamage,
    NodeFlags, NodeTraits, ShadowIncluding, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::promise::Promise;
use crate::dom::raredata::ElementRareData;
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::{IsUserAgentWidget, ShadowRoot};
use crate::dom::text::Text;
use crate::dom::trustedhtml::TrustedHTML;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::{VirtualMethods, vtable_for};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;
use crate::stylesheet_loader::StylesheetOwner;
use crate::task::TaskOnce;

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
    style_attribute: DomRefCell<Option<Arc<Locked<PropertyDeclarationBlock>>>>,
    attr_list: MutNullableDom<NamedNodeMap>,
    class_list: MutNullableDom<DOMTokenList>,
    #[no_trace]
    state: Cell<ElementState>,
    /// These flags are set by the style system to indicate the that certain
    /// operations may require restyling this element or its descendants. The
    /// flags are not atomic, so the style system takes care of only set them
    /// when it has exclusive access to the element.
    #[ignore_malloc_size_of = "bitflags defined in rust-selectors"]
    #[no_trace]
    selector_flags: Cell<ElementSelectorFlags>,
    rare_data: DomRefCell<Option<Box<ElementRareData>>>,
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
            _             => Err(Error::Syntax)
        }
    }
}

//
// Element methods
//
impl Element {
    pub(crate) fn create(
        name: QualName,
        is: Option<LocalName>,
        document: &Document,
        creator: ElementCreator,
        mode: CustomElementCreationMode,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Element> {
        create_element(name, is, document, creator, mode, proto, can_gc)
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
            selector_flags: Cell::new(ElementSelectorFlags::empty()),
            rare_data: Default::default(),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Element> {
        Node::reflect_node_with_proto(
            Box::new(Element::new_inherited(
                local_name, namespace, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    fn rare_data(&self) -> Ref<Option<Box<ElementRareData>>> {
        self.rare_data.borrow()
    }

    fn rare_data_mut(&self) -> RefMut<Option<Box<ElementRareData>>> {
        self.rare_data.borrow_mut()
    }

    fn ensure_rare_data(&self) -> RefMut<Box<ElementRareData>> {
        let mut rare_data = self.rare_data.borrow_mut();
        if rare_data.is_none() {
            *rare_data = Some(Default::default());
        }
        RefMut::map(rare_data, |rare_data| rare_data.as_mut().unwrap())
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
                    .insert(LayoutDamage::recollect_box_tree_children());
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

    /// <https://dom.spec.whatwg.org/#concept-element-custom-element-state>
    pub(crate) fn set_custom_element_state(&self, state: CustomElementState) {
        // no need to inflate rare data for uncustomized
        if state != CustomElementState::Uncustomized || self.rare_data().is_some() {
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

    pub(crate) fn invoke_reactions(&self, can_gc: CanGc) {
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
                reaction.invoke(self, can_gc);
            }

            reactions.clear();
        }
    }

    /// style will be `None` for elements in a `display: none` subtree. otherwise, the element has a
    /// layout box iff it doesn't have `display: none`.
    pub(crate) fn style(&self, can_gc: CanGc) -> Option<Arc<ComputedValues>> {
        self.upcast::<Node>().style(can_gc)
    }

    // https://drafts.csswg.org/cssom-view/#css-layout-box
    pub(crate) fn has_css_layout_box(&self, can_gc: CanGc) -> bool {
        self.style(can_gc)
            .is_some_and(|s| !s.get_box().clone_display().is_none())
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    pub(crate) fn is_potentially_scrollable_body(&self, can_gc: CanGc) -> bool {
        self.is_potentially_scrollable_body_shared_logic(false, can_gc)
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    pub(crate) fn is_potentially_scrollable_body_for_scrolling_element(
        &self,
        can_gc: CanGc,
    ) -> bool {
        self.is_potentially_scrollable_body_shared_logic(true, can_gc)
    }

    /// <https://drafts.csswg.org/cssom-view/#potentially-scrollable>
    fn is_potentially_scrollable_body_shared_logic(
        &self,
        treat_overflow_clip_on_parent_as_hidden: bool,
        can_gc: CanGc,
    ) -> bool {
        let node = self.upcast::<Node>();
        debug_assert!(
            node.owner_doc().GetBody().as_deref() == self.downcast::<HTMLElement>(),
            "Called is_potentially_scrollable_body on element that is not the <body>"
        );

        // "An element body (which will be the body element) is potentially
        // scrollable if all of the following conditions are true:
        //  - body has an associated box."
        if !self.has_css_layout_box(can_gc) {
            return false;
        }

        // " - body’s parent element’s computed value of the overflow-x or
        //     overflow-y properties is neither visible nor clip."
        if let Some(parent) = node.GetParentElement() {
            if let Some(style) = parent.style(can_gc) {
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
        if let Some(style) = self.style(can_gc) {
            if !style.get_box().clone_overflow_x().is_scrollable() &&
                !style.get_box().clone_overflow_y().is_scrollable()
            {
                return false;
            }
        };

        true
    }

    // https://drafts.csswg.org/cssom-view/#scrolling-box
    fn has_scrolling_box(&self, can_gc: CanGc) -> bool {
        // TODO: scrolling mechanism, such as scrollbar (We don't have scrollbar yet)
        //       self.has_scrolling_mechanism()
        self.style(can_gc).is_some_and(|style| {
            style.get_box().clone_overflow_x().is_scrollable() ||
                style.get_box().clone_overflow_y().is_scrollable()
        })
    }

    fn has_overflow(&self, can_gc: CanGc) -> bool {
        self.ScrollHeight(can_gc) > self.ClientHeight(can_gc) ||
            self.ScrollWidth(can_gc) > self.ClientWidth(can_gc)
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
        is_ua_widget: IsUserAgentWidget,
        mode: ShadowRootMode,
        clonable: bool,
        serializable: bool,
        delegates_focus: bool,
        slot_assignment_mode: SlotAssignmentMode,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1. If element’s namespace is not the HTML namespace,
        // then throw a "NotSupportedError" DOMException.
        if self.namespace != ns!(html) {
            return Err(Error::NotSupported);
        }

        // Step 2. If element’s local name is not a valid shadow host name,
        // then throw a "NotSupportedError" DOMException.
        if !is_valid_shadow_host_name(self.local_name()) {
            // UA shadow roots may be attached to anything
            if is_ua_widget != IsUserAgentWidget::Yes {
                return Err(Error::NotSupported);
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
                return Err(Error::NotSupported);
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
                return Err(Error::NotSupported);
            }

            // Step 4.3.1. Remove all of currentShadowRoot’s children, in tree order.
            for child in current_shadow_root.upcast::<Node>().children() {
                child.remove_self(can_gc);
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
            can_gc,
        );

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

        let bind_context = BindContext {
            tree_connected: self.upcast::<Node>().is_connected(),
            tree_is_in_a_document_tree: self.upcast::<Node>().is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: true,
        };
        shadow_root.bind_to_tree(&bind_context, can_gc);

        let node = self.upcast::<Node>();
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
        use_ua_widget_styling: bool,
        can_gc: CanGc,
    ) -> DomRoot<ShadowRoot> {
        let root = self
            .attach_shadow(
                IsUserAgentWidget::Yes,
                ShadowRootMode::Closed,
                false,
                false,
                false,
                SlotAssignmentMode::Manual,
                can_gc,
            )
            .expect("Attaching UA shadow root failed");

        root.upcast::<Node>()
            .set_in_ua_widget(use_ua_widget_styling);
        root
    }

    pub(crate) fn detach_shadow(&self, can_gc: CanGc) {
        let Some(ref shadow_root) = self.shadow_root() else {
            unreachable!("Trying to detach a non-attached shadow root");
        };

        let node = self.upcast::<Node>();
        node.note_dirty_descendants();
        node.rev_version();

        shadow_root.detach(can_gc);
        self.ensure_rare_data().shadow_root = None;
    }

    // https://html.spec.whatwg.org/multipage/#translation-mode
    pub(crate) fn is_translate_enabled(&self) -> bool {
        let name = &html5ever::local_name!("translate");
        if self.has_attribute(name) {
            match_ignore_ascii_case! { &*self.get_string_attribute(name),
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
    ) -> RefMut<Vec<IntersectionObserverRegistration>> {
        RefMut::map(self.ensure_rare_data(), |rare_data| {
            &mut rare_data.registered_intersection_observers
        })
    }

    pub(crate) fn registered_intersection_observers(
        &self,
    ) -> Option<Ref<Vec<IntersectionObserverRegistration>>> {
        let rare_data: Ref<_> = self.rare_data.borrow();

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
    ) -> Option<Ref<IntersectionObserverRegistration>> {
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

    /// <https://html.spec.whatwg.org/multipage/#matches-the-environment>
    pub(crate) fn matches_environment(&self, media_query: &str) -> bool {
        let document = self.owner_document();
        let quirks_mode = document.quirks_mode();
        let document_url_data = UrlExtraData(document.url().get_arc());
        // FIXME(emilio): This should do the same that we do for other media
        // lists regarding the rule type and such, though it doesn't really
        // matter right now...
        //
        // Also, ParsingMode::all() is wrong, and should be DEFAULT.
        let context = CssParserContext::new(
            CssOrigin::Author,
            &document_url_data,
            Some(CssRuleType::Style),
            CssParsingMode::all(),
            quirks_mode,
            /* namespaces = */ Default::default(),
            None,
            None,
        );
        let mut parser_input = CssParserInput::new(media_query);
        let mut parser = CssParser::new(&mut parser_input);
        let media_list = MediaList::parse(&context, &mut parser);
        let result = media_list.evaluate(document.window().layout().device(), quirks_mode);
        result
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
) -> Option<LayoutDom<'dom, Attr>> {
    elem.attrs()
        .iter()
        .find(|attr| name == attr.local_name() && namespace == attr.namespace())
        .cloned()
}

pub(crate) trait LayoutElementHelpers<'dom> {
    fn attrs(self) -> &'dom [LayoutDom<'dom, Attr>];
    fn has_class_or_part_for_layout(
        self,
        name: &AtomIdent,
        attr_name: &LocalName,
        case_sensitivity: CaseSensitivity,
    ) -> bool;
    fn get_classes_for_layout(self) -> Option<&'dom [Atom]>;
    fn get_parts_for_layout(self) -> Option<&'dom [Atom]>;

    fn synthesize_presentational_hints_for_legacy_attributes<V>(self, hints: &mut V)
    where
        V: Push<ApplicableDeclarationBlock>;
    fn get_span(self) -> Option<u32>;
    fn get_colspan(self) -> Option<u32>;
    fn get_rowspan(self) -> Option<u32>;
    fn is_html_element(&self) -> bool;
    fn id_attribute(self) -> *const Option<Atom>;
    fn style_attribute(self) -> *const Option<Arc<Locked<PropertyDeclarationBlock>>>;
    fn local_name(self) -> &'dom LocalName;
    fn namespace(self) -> &'dom Namespace;
    fn get_lang_attr_val_for_layout(self) -> Option<&'dom str>;
    fn get_lang_for_layout(self) -> String;
    fn get_state_for_layout(self) -> ElementState;
    fn insert_selector_flags(self, flags: ElementSelectorFlags);
    fn get_selector_flags(self) -> ElementSelectorFlags;
    /// The shadow root this element is a host of.
    fn get_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>>;
    fn get_attr_for_layout(
        self,
        namespace: &Namespace,
        name: &LocalName,
    ) -> Option<&'dom AttrValue>;
    fn get_attr_val_for_layout(self, namespace: &Namespace, name: &LocalName) -> Option<&'dom str>;
    fn get_attr_vals_for_layout(self, name: &LocalName) -> Vec<&'dom AttrValue>;
}

impl LayoutDom<'_, Element> {
    pub(super) fn focus_state(self) -> bool {
        self.unsafe_get().state.get().contains(ElementState::FOCUS)
    }
}

impl<'dom> LayoutElementHelpers<'dom> for LayoutDom<'dom, Element> {
    #[allow(unsafe_code)]
    #[inline]
    fn attrs(self) -> &'dom [LayoutDom<'dom, Attr>] {
        unsafe { LayoutDom::to_layout_slice(self.unsafe_get().attrs.borrow_for_layout()) }
    }

    #[inline]
    fn has_class_or_part_for_layout(
        self,
        name: &AtomIdent,
        attr_name: &LocalName,
        case_sensitivity: CaseSensitivity,
    ) -> bool {
        get_attr_for_layout(self, &ns!(), attr_name).is_some_and(|attr| {
            attr.to_tokens()
                .unwrap()
                .iter()
                .any(|atom| case_sensitivity.eq_atom(atom, name))
        })
    }

    #[inline]
    fn get_classes_for_layout(self) -> Option<&'dom [Atom]> {
        get_attr_for_layout(self, &ns!(), &local_name!("class"))
            .map(|attr| attr.to_tokens().unwrap())
    }

    fn get_parts_for_layout(self) -> Option<&'dom [Atom]> {
        get_attr_for_layout(self, &ns!(), &local_name!("part"))
            .map(|attr| attr.to_tokens().unwrap())
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(self, hints: &mut V)
    where
        V: Push<ApplicableDeclarationBlock>,
    {
        // FIXME(emilio): Just a single PDB should be enough.
        #[inline]
        fn from_declaration(
            shared_lock: &SharedRwLock,
            declaration: PropertyDeclaration,
        ) -> ApplicableDeclarationBlock {
            ApplicableDeclarationBlock::from_declarations(
                Arc::new(shared_lock.wrap(PropertyDeclarationBlock::with_one(
                    declaration,
                    Importance::Normal,
                ))),
                CascadeLevel::PresHints,
                LayerOrder::root(),
            )
        }

        let document = self.upcast::<Node>().owner_doc_for_layout();
        let shared_lock = document.style_shared_lock();

        // TODO(xiaochengh): This is probably not enough. When the root element doesn't have a `lang`,
        // we should check the browser settings and system locale.
        if let Some(lang) = self.get_lang_attr_val_for_layout() {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::XLang(specified::XLang(Atom::from(lang.to_owned()))),
            ));
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
                shared_lock,
                PropertyDeclaration::BackgroundColor(specified::Color::from_absolute_color(color)),
            ));
        }

        let background = if let Some(this) = self.downcast::<HTMLBodyElement>() {
            this.get_background()
        } else {
            None
        };

        if let Some(url) = background {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BackgroundImage(background_image::SpecifiedValue(
                    vec![specified::Image::for_cascade(url.get_arc())].into(),
                )),
            ));
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
                PropertyDeclaration::Color(longhands::color::SpecifiedValue(
                    specified::Color::from_absolute_color(color),
                )),
            ));
        }

        let font_face = if let Some(this) = self.downcast::<HTMLFontElement>() {
            this.get_face()
        } else {
            None
        };

        if let Some(font_face) = font_face {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::FontFamily(font_family::SpecifiedValue::Values(
                    computed::font::FontFamilyList {
                        list: ArcSlice::from_iter(
                            HTMLFontElement::parse_face_attribute(font_face).into_iter(),
                        ),
                    },
                )),
            ));
        }

        let font_size = self
            .downcast::<HTMLFontElement>()
            .and_then(|this| this.get_size());

        if let Some(font_size) = font_size {
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::FontSize(font_size::SpecifiedValue::from_html_size(
                    font_size as u8,
                )),
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
                PropertyDeclaration::BorderSpacing(Box::new(border_spacing::SpecifiedValue::new(
                    width_value.clone().into(),
                    width_value.into(),
                ))),
            ));
        }

        let size = if let Some(this) = self.downcast::<HTMLInputElement>() {
            // FIXME(pcwalton): More use of atoms, please!
            match self.get_attr_val_for_layout(&ns!(), &local_name!("type")) {
                // Not text entry widget
                Some("hidden") |
                Some("date") |
                Some("month") |
                Some("week") |
                Some("time") |
                Some("datetime-local") |
                Some("number") |
                Some("range") |
                Some("color") |
                Some("checkbox") |
                Some("radio") |
                Some("file") |
                Some("submit") |
                Some("image") |
                Some("reset") |
                Some("button") => None,
                // Others
                _ => match this.size_for_layout() {
                    0 => None,
                    s => Some(s as i32),
                },
            }
        } else {
            None
        };

        if let Some(size) = size {
            let value =
                specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(size));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Width(specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(value),
                ))),
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
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Width(width_value),
                ));
            },
            LengthOrPercentageOrAuto::Length(length) => {
                let width_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(specified::NoCalcLength::Absolute(
                        specified::AbsoluteLength::Px(length.to_f32_px()),
                    )),
                ));
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Width(width_value),
                ));
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
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Height(height_value),
                ));
            },
            LengthOrPercentageOrAuto::Length(length) => {
                let height_value = specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(specified::NoCalcLength::Absolute(
                        specified::AbsoluteLength::Px(length.to_f32_px()),
                    )),
                ));
                hints.push(from_declaration(
                    shared_lock,
                    PropertyDeclaration::Height(height_value),
                ));
            },
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
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::AspectRatio(aspect_ratio),
                    ));
                }
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
            let value =
                specified::NoCalcLength::ServoCharacterWidth(specified::CharacterWidth(cols));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Width(specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(value),
                ))),
            ));
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
            let value = specified::NoCalcLength::FontRelative(specified::FontRelativeLength::Em(
                rows as CSSFloat,
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::Height(specified::Size::LengthPercentage(NonNegative(
                    specified::LengthPercentage::Length(value),
                ))),
            ));
        }

        let border = if let Some(this) = self.downcast::<HTMLTableElement>() {
            this.get_border()
        } else {
            None
        };

        if let Some(border) = border {
            let width_value = specified::BorderSideWidth::from_px(border as f32);
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderTopWidth(width_value.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderLeftWidth(width_value.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderBottomWidth(width_value.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::BorderRightWidth(width_value),
            ));
        }

        if let Some(cellpadding) = self
            .downcast::<HTMLTableCellElement>()
            .and_then(|this| this.get_table())
            .and_then(|table| table.get_cellpadding())
        {
            let cellpadding = NonNegative(specified::LengthPercentage::Length(
                specified::NoCalcLength::from_px(cellpadding as f32),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::PaddingTop(cellpadding.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::PaddingLeft(cellpadding.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::PaddingBottom(cellpadding.clone()),
            ));
            hints.push(from_declaration(
                shared_lock,
                PropertyDeclaration::PaddingRight(cellpadding),
            ));
        }

        // https://html.spec.whatwg.org/multipage/#the-hr-element-2
        if let Some(size_info) = self
            .downcast::<HTMLHRElement>()
            .and_then(|hr_element| hr_element.get_size_info())
        {
            match size_info {
                SizePresentationalHint::SetHeightTo(height) => {
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::Height(height),
                    ));
                },
                SizePresentationalHint::SetAllBorderWidthValuesTo(border_width) => {
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::BorderLeftWidth(border_width.clone()),
                    ));
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::BorderRightWidth(border_width.clone()),
                    ));
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::BorderTopWidth(border_width.clone()),
                    ));
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::BorderBottomWidth(border_width),
                    ));
                },
                SizePresentationalHint::SetBottomBorderWidthToZero => {
                    hints.push(from_declaration(
                        shared_lock,
                        PropertyDeclaration::BorderBottomWidth(
                            specified::border::BorderSideWidth::from_px(0.),
                        ),
                    ));
                },
            }
        }
    }

    fn get_span(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableColElement>()
            .and_then(|element| element.get_span())
    }

    fn get_colspan(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableCellElement>()
            .and_then(|element| element.get_colspan())
    }

    fn get_rowspan(self) -> Option<u32> {
        // Don't panic since `display` can cause this to be called on arbitrary elements.
        self.downcast::<HTMLTableCellElement>()
            .and_then(|element| element.get_rowspan())
    }

    #[inline]
    fn is_html_element(&self) -> bool {
        *self.namespace() == ns!(html)
    }

    #[allow(unsafe_code)]
    fn id_attribute(self) -> *const Option<Atom> {
        unsafe { (self.unsafe_get()).id_attribute.borrow_for_layout() }
    }

    #[allow(unsafe_code)]
    fn style_attribute(self) -> *const Option<Arc<Locked<PropertyDeclarationBlock>>> {
        unsafe { (self.unsafe_get()).style_attribute.borrow_for_layout() }
    }

    #[allow(unsafe_code)]
    fn local_name(self) -> &'dom LocalName {
        &(self.unsafe_get()).local_name
    }

    fn namespace(self) -> &'dom Namespace {
        &(self.unsafe_get()).namespace
    }

    fn get_lang_attr_val_for_layout(self) -> Option<&'dom str> {
        if let Some(attr) = self.get_attr_val_for_layout(&ns!(xml), &local_name!("lang")) {
            return Some(attr);
        }
        if let Some(attr) = self.get_attr_val_for_layout(&ns!(), &local_name!("lang")) {
            return Some(attr);
        }
        None
    }

    fn get_lang_for_layout(self) -> String {
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
    fn get_state_for_layout(self) -> ElementState {
        (self.unsafe_get()).state.get()
    }

    #[inline]
    fn insert_selector_flags(self, flags: ElementSelectorFlags) {
        debug_assert!(thread_state::get().is_layout());
        let f = &(self.unsafe_get()).selector_flags;
        f.set(f.get() | flags);
    }

    #[inline]
    fn get_selector_flags(self) -> ElementSelectorFlags {
        self.unsafe_get().selector_flags.get()
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>> {
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
    fn get_attr_for_layout(
        self,
        namespace: &Namespace,
        name: &LocalName,
    ) -> Option<&'dom AttrValue> {
        get_attr_for_layout(self, namespace, name).map(|attr| attr.value())
    }

    #[inline]
    fn get_attr_val_for_layout(self, namespace: &Namespace, name: &LocalName) -> Option<&'dom str> {
        get_attr_for_layout(self, namespace, name).map(|attr| attr.as_str())
    }

    #[inline]
    fn get_attr_vals_for_layout(self, name: &LocalName) -> Vec<&'dom AttrValue> {
        self.attrs()
            .iter()
            .filter_map(|attr| {
                if name == attr.local_name() {
                    Some(attr.value())
                } else {
                    None
                }
            })
            .collect()
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

    pub(crate) fn prefix(&self) -> Ref<Option<Prefix>> {
        self.prefix.borrow()
    }

    pub(crate) fn set_prefix(&self, prefix: Option<Prefix>) {
        *self.prefix.borrow_mut() = prefix;
    }

    pub(crate) fn attrs(&self) -> Ref<[Dom<Attr>]> {
        Ref::map(self.attrs.borrow(), |attrs| &**attrs)
    }

    // Element branch of https://dom.spec.whatwg.org/#locate-a-namespace
    pub(crate) fn locate_namespace(&self, prefix: Option<DOMString>) -> Namespace {
        let namespace_prefix = prefix.clone().map(|s| Prefix::from(&*s));

        // "1. If prefix is "xml", then return the XML namespace."
        if namespace_prefix == Some(namespace_prefix!("xml")) {
            return ns!(xml);
        }

        // "2. If prefix is "xmlns", then return the XMLNS namespace."
        if namespace_prefix == Some(namespace_prefix!("xmlns")) {
            return ns!(xmlns);
        }

        let prefix = prefix.map(|s| LocalName::from(&*s));

        let inclusive_ancestor_elements = self
            .upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<Self>);

        // "5. If its parent element is null, then return null."
        // "6. Return the result of running locate a namespace on its parent element using prefix."
        for element in inclusive_ancestor_elements {
            // "3. If its namespace is non-null and its namespace prefix is prefix, then return
            // namespace."
            if element.namespace() != &ns!() &&
                element.prefix().as_ref().map(|p| &**p) == prefix.as_deref()
            {
                return element.namespace().clone();
            }

            // "4. If it has an attribute whose namespace is the XMLNS namespace, namespace prefix
            // is "xmlns", and local name is prefix, or if prefix is null and it has an attribute
            // whose namespace is the XMLNS namespace, namespace prefix is null, and local name is
            // "xmlns", then return its value if it is not the empty string, and null otherwise."
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
    ) -> &DomRefCell<Option<Arc<Locked<PropertyDeclarationBlock>>>> {
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

    // Returns the kind of IME control needed for a focusable element, if any.
    pub(crate) fn input_method_type(&self) -> Option<InputMethodType> {
        if !self.is_focusable_area() {
            return None;
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() {
            input.input_type().as_ime_type()
        } else if self.is::<HTMLTextAreaElement>() {
            Some(InputMethodType::Text)
        } else {
            // Other focusable elements that are not input fields.
            None
        }
    }

    pub(crate) fn is_focusable_area(&self) -> bool {
        if self.is_actually_disabled() {
            return false;
        }
        let node = self.upcast::<Node>();
        if node.get_flag(NodeFlags::SEQUENTIALLY_FOCUSABLE) {
            return true;
        }

        // <a>, <input>, <select>, and <textrea> are inherently focusable.
        matches!(
            node.type_id(),
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) | NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLTextAreaElement,
            ))
        )
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

    pub(crate) fn push_new_attribute(
        &self,
        local_name: LocalName,
        value: AttrValue,
        name: LocalName,
        namespace: Namespace,
        prefix: Option<Prefix>,
        can_gc: CanGc,
    ) {
        let attr = Attr::new(
            &self.node.owner_doc(),
            local_name,
            value,
            name,
            namespace,
            prefix,
            Some(self),
            can_gc,
        );
        self.push_attribute(&attr, can_gc);
    }

    pub(crate) fn push_attribute(&self, attr: &Attr, can_gc: CanGc) {
        let name = attr.local_name().clone();
        let namespace = attr.namespace().clone();
        let mutation = LazyCell::new(|| Mutation::Attribute {
            name: name.clone(),
            namespace: namespace.clone(),
            old_value: None,
        });

        MutationObserver::queue_a_mutation_record(&self.node, mutation);

        if self.is_custom() {
            let value = DOMString::from(&**attr.value());
            let reaction = CallbackReaction::AttributeChanged(name, None, Some(value), namespace);
            ScriptThread::enqueue_callback_reaction(self, reaction, None);
        }

        assert!(attr.GetOwnerElement().as_deref() == Some(self));
        self.will_mutate_attr(attr);
        self.attrs.borrow_mut().push(Dom::from_ref(attr));
        if is_relevant_attribute(attr.namespace(), attr.local_name()) {
            vtable_for(self.upcast()).attribute_mutated(attr, AttributeMutation::Set(None), can_gc);
        }
    }

    pub(crate) fn get_attribute(
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
        self.push_new_attribute(qname.local, value, name, qname.ns, prefix, can_gc);
    }

    pub(crate) fn set_attribute(&self, name: &LocalName, value: AttrValue, can_gc: CanGc) {
        assert!(name == &name.to_ascii_lowercase());
        assert!(!name.contains(':'));

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

    // https://html.spec.whatwg.org/multipage/#attr-data-*
    pub(crate) fn set_custom_attribute(
        &self,
        name: DOMString,
        value: DOMString,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 1.
        if !matches_name_production(&name) {
            return Err(Error::InvalidCharacter);
        }

        // Steps 2-5.
        let name = LocalName::from(name);
        let value = self.parse_attribute(&ns!(), &name, value);
        self.set_first_matching_attribute(
            name.clone(),
            value,
            name.clone(),
            ns!(),
            None,
            |attr| *attr.name() == name && *attr.namespace() == ns!(),
            can_gc,
        );
        Ok(())
    }

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
        let attr = self
            .attrs
            .borrow()
            .iter()
            .find(|attr| find(attr))
            .map(|js| DomRoot::from_ref(&**js));
        if let Some(attr) = attr {
            attr.set_value(value, self, can_gc);
        } else {
            self.push_new_attribute(local_name, value, name, namespace, prefix, can_gc);
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

    fn remove_first_matching_attribute<F>(&self, find: F, can_gc: CanGc) -> Option<DomRoot<Attr>>
    where
        F: Fn(&Attr) -> bool,
    {
        let idx = self.attrs.borrow().iter().position(|attr| find(attr));
        idx.map(|idx| {
            let attr = DomRoot::from_ref(&*(*self.attrs.borrow())[idx]);
            self.will_mutate_attr(&attr);

            let name = attr.local_name().clone();
            let namespace = attr.namespace().clone();
            let old_value = DOMString::from(&**attr.value());
            let mutation = LazyCell::new(|| Mutation::Attribute {
                name: name.clone(),
                namespace: namespace.clone(),
                old_value: Some(old_value.clone()),
            });

            MutationObserver::queue_a_mutation_record(&self.node, mutation);

            if self.is_custom() {
                let reaction =
                    CallbackReaction::AttributeChanged(name, Some(old_value), None, namespace);
                ScriptThread::enqueue_callback_reaction(self, reaction, None);
            }

            self.attrs.borrow_mut().remove(idx);
            attr.set_owner(None);
            if is_relevant_attribute(attr.namespace(), attr.local_name()) {
                vtable_for(self.upcast()).attribute_mutated(
                    &attr,
                    AttributeMutation::Removed,
                    can_gc,
                );
            }
            attr
        })
    }

    pub(crate) fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        self.get_attribute(&ns!(), &local_name!("class"))
            .is_some_and(|attr| {
                attr.value()
                    .as_tokens()
                    .iter()
                    .any(|atom| case_sensitivity.eq_atom(name, atom))
            })
    }

    pub(crate) fn is_part(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        self.get_attribute(&ns!(), &LocalName::from("part"))
            .is_some_and(|attr| {
                attr.value()
                    .as_tokens()
                    .iter()
                    .any(|atom| case_sensitivity.eq_atom(name, atom))
            })
    }

    pub(crate) fn set_atomic_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        let value = AttrValue::from_atomic(value.into());
        self.set_attribute(local_name, value, can_gc);
    }

    pub(crate) fn has_attribute(&self, local_name: &LocalName) -> bool {
        assert!(local_name.bytes().all(|b| b.to_ascii_lowercase() == b));
        self.attrs
            .borrow()
            .iter()
            .any(|attr| attr.local_name() == local_name && attr.namespace() == &ns!())
    }

    pub(crate) fn set_bool_attribute(&self, local_name: &LocalName, value: bool, can_gc: CanGc) {
        if self.has_attribute(local_name) == value {
            return;
        }
        if value {
            self.set_string_attribute(local_name, DOMString::new(), can_gc);
        } else {
            self.remove_attribute(&ns!(), local_name, can_gc);
        }
    }

    pub(crate) fn get_url_attribute(&self, local_name: &LocalName) -> USVString {
        assert!(*local_name == local_name.to_ascii_lowercase());
        let attr = match self.get_attribute(&ns!(), local_name) {
            Some(attr) => attr,
            None => return USVString::default(),
        };
        let value = &**attr.value();
        // XXXManishearth this doesn't handle `javascript:` urls properly
        self.owner_document()
            .base_url()
            .join(value)
            .map(|parsed| USVString(parsed.into_string()))
            .unwrap_or_else(|_| USVString(value.to_owned()))
    }

    pub(crate) fn set_url_attribute(
        &self,
        local_name: &LocalName,
        value: USVString,
        can_gc: CanGc,
    ) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value.to_string()), can_gc);
    }

    pub(crate) fn get_trusted_type_url_attribute(
        &self,
        local_name: &LocalName,
    ) -> TrustedScriptURLOrUSVString {
        assert_eq!(*local_name, local_name.to_ascii_lowercase());
        let attr = match self.get_attribute(&ns!(), local_name) {
            Some(attr) => attr,
            None => return TrustedScriptURLOrUSVString::USVString(USVString::default()),
        };
        let value = &**attr.value();
        // XXXManishearth this doesn't handle `javascript:` urls properly
        self.owner_document()
            .base_url()
            .join(value)
            .map(|parsed| TrustedScriptURLOrUSVString::USVString(USVString(parsed.into_string())))
            .unwrap_or_else(|_| TrustedScriptURLOrUSVString::USVString(USVString(value.to_owned())))
    }

    pub(crate) fn get_trusted_html_attribute(&self, local_name: &LocalName) -> TrustedHTMLOrString {
        assert_eq!(*local_name, local_name.to_ascii_lowercase());
        let value = match self.get_attribute(&ns!(), local_name) {
            Some(attr) => (&**attr.value()).into(),
            None => "".into(),
        };
        TrustedHTMLOrString::String(value)
    }

    pub(crate) fn get_string_attribute(&self, local_name: &LocalName) -> DOMString {
        match self.get_attribute(&ns!(), local_name) {
            Some(x) => x.Value(),
            None => DOMString::new(),
        }
    }

    pub(crate) fn set_string_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::String(value.into()), can_gc);
    }

    /// Used for string attribute reflections where absence of the attribute returns `null`,
    /// e.g. `element.ariaLabel` returning `null` when the `aria-label` attribute is absent.
    fn get_nullable_string_attribute(&self, local_name: &LocalName) -> Option<DOMString> {
        if self.has_attribute(local_name) {
            Some(self.get_string_attribute(local_name))
        } else {
            None
        }
    }

    /// Used for string attribute reflections where setting `null`/`undefined` removes the
    /// attribute, e.g. `element.ariaLabel = null` removing the `aria-label` attribute.
    fn set_nullable_string_attribute(
        &self,
        local_name: &LocalName,
        value: Option<DOMString>,
        can_gc: CanGc,
    ) {
        match value {
            Some(val) => {
                self.set_string_attribute(local_name, val, can_gc);
            },
            None => {
                self.remove_attribute(&ns!(), local_name, can_gc);
            },
        }
    }

    pub(crate) fn get_tokenlist_attribute(&self, local_name: &LocalName) -> Vec<Atom> {
        self.get_attribute(&ns!(), local_name)
            .map(|attr| attr.value().as_tokens().to_vec())
            .unwrap_or_default()
    }

    pub(crate) fn set_tokenlist_attribute(
        &self,
        local_name: &LocalName,
        value: DOMString,
        can_gc: CanGc,
    ) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(
            local_name,
            AttrValue::from_serialized_tokenlist(value.into()),
            can_gc,
        );
    }

    pub(crate) fn set_atomic_tokenlist_attribute(
        &self,
        local_name: &LocalName,
        tokens: Vec<Atom>,
        can_gc: CanGc,
    ) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::from_atomic_tokens(tokens), can_gc);
    }

    pub(crate) fn get_int_attribute(&self, local_name: &LocalName, default: i32) -> i32 {
        // TODO: Is this assert necessary?
        assert!(
            local_name
                .chars()
                .all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch)
        );
        let attribute = self.get_attribute(&ns!(), local_name);

        match attribute {
            Some(ref attribute) => match *attribute.value() {
                AttrValue::Int(_, value) => value,
                _ => panic!(
                    "Expected an AttrValue::Int: \
                     implement parse_plain_attribute"
                ),
            },
            None => default,
        }
    }

    pub(crate) fn set_int_attribute(&self, local_name: &LocalName, value: i32, can_gc: CanGc) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(local_name, AttrValue::Int(value.to_string(), value), can_gc);
    }

    pub(crate) fn get_uint_attribute(&self, local_name: &LocalName, default: u32) -> u32 {
        assert!(
            local_name
                .chars()
                .all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch)
        );
        let attribute = self.get_attribute(&ns!(), local_name);
        match attribute {
            Some(ref attribute) => match *attribute.value() {
                AttrValue::UInt(_, value) => value,
                _ => panic!("Expected an AttrValue::UInt: implement parse_plain_attribute"),
            },
            None => default,
        }
    }
    pub(crate) fn set_uint_attribute(&self, local_name: &LocalName, value: u32, can_gc: CanGc) {
        assert!(*local_name == local_name.to_ascii_lowercase());
        self.set_attribute(
            local_name,
            AttrValue::UInt(value.to_string(), value),
            can_gc,
        );
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
                        )
                    {
                        return;
                    }
                    Arc::new(doc.style_shared_lock().wrap(parse_style_attribute(
                        source,
                        &UrlExtraData(doc.base_url().get_arc()),
                        win.css_error_reporter(),
                        doc.quirks_mode(),
                        CssRuleType::Style,
                    )))
                };

                Some(block)
            },
            AttributeMutation::Removed => None,
        };
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
        self.set_string_attribute(&local_name!("nonce"), "".into(), CanGc::note());
        // Step 2.3: Set element's [[CryptographicNonce]] to nonce.
        self.update_nonce_internal_slot(nonce);
    }

    /// <https://www.w3.org/TR/CSP/#is-element-nonceable>
    pub(crate) fn nonce_value_if_nonceable(&self) -> Option<String> {
        // Step 1: If element does not have an attribute named "nonce", return "Not Nonceable".
        if !self.has_attribute(&local_name!("nonce")) {
            return None;
        }
        // Step 2: If element is a script element, then for each attribute of element’s attribute list:
        if self.downcast::<HTMLScriptElement>().is_some() {
            for attr in self.attrs().iter() {
                // Step 2.1: If attribute’s name contains an ASCII case-insensitive match
                // for "<script" or "<style", return "Not Nonceable".
                let attr_name = attr.name().to_ascii_lowercase();
                if attr_name.contains("<script") || attr_name.contains("<style") {
                    return None;
                }
                // Step 2.2: If attribute’s value contains an ASCII case-insensitive match
                // for "<script" or "<style", return "Not Nonceable".
                let attr_value = attr.value().to_ascii_lowercase();
                if attr_value.contains("<script") || attr_value.contains("<style") {
                    return None;
                }
            }
        }
        // Step 3: If element had a duplicate-attribute parse error during tokenization, return "Not Nonceable".
        // TODO(https://github.com/servo/servo/issues/4577 and https://github.com/whatwg/html/issues/3257):
        // Figure out how to retrieve this information from the parser
        // Step 4: Return "Nonceable".
        Some(self.nonce_value().trim().to_owned())
    }

    // https://dom.spec.whatwg.org/#insert-adjacent
    pub(crate) fn insert_adjacent(
        &self,
        where_: AdjacentPosition,
        node: &Node,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<Node>>> {
        let self_node = self.upcast::<Node>();
        match where_ {
            AdjacentPosition::BeforeBegin => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, Some(self_node), can_gc).map(Some)
                } else {
                    Ok(None)
                }
            },
            AdjacentPosition::AfterBegin => Node::pre_insert(
                node,
                self_node,
                self_node.GetFirstChild().as_deref(),
                can_gc,
            )
            .map(Some),
            AdjacentPosition::BeforeEnd => {
                Node::pre_insert(node, self_node, None, can_gc).map(Some)
            },
            AdjacentPosition::AfterEnd => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, self_node.GetNextSibling().as_deref(), can_gc)
                        .map(Some)
                } else {
                    Ok(None)
                }
            },
        }
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    pub(crate) fn scroll(&self, x_: f64, y_: f64, behavior: ScrollBehavior, can_gc: CanGc) {
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
                win.scroll(x, y, behavior, can_gc);
            }

            return;
        }

        // Step 9
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body(can_gc)
        {
            win.scroll(x, y, behavior, can_gc);
            return;
        }

        // Step 10
        if !self.has_css_layout_box(can_gc) ||
            !self.has_scrolling_box(can_gc) ||
            !self.has_overflow(can_gc)
        {
            return;
        }

        // Step 11
        win.scroll_node(node, x, y, behavior, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#fragment-parsing-algorithm-steps>
    pub(crate) fn parse_fragment(
        &self,
        markup: DOMString,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<DocumentFragment>> {
        // Steps 1-2.
        // TODO(#11995): XML case.
        let new_children = ServoParser::parse_html_fragment(self, markup, false, can_gc);
        // Step 3.
        // See https://github.com/w3c/DOM-Parsing/issues/61.
        let context_document = {
            if let Some(template) = self.downcast::<HTMLTemplateElement>() {
                template.Content(can_gc).upcast::<Node>().owner_doc()
            } else {
                self.owner_document()
            }
        };
        let fragment = DocumentFragment::new(&context_document, can_gc);
        // Step 4.
        for child in new_children {
            fragment
                .upcast::<Node>()
                .AppendChild(&child, can_gc)
                .unwrap();
        }
        // Step 5.
        Ok(fragment)
    }

    /// Step 4 of <https://html.spec.whatwg.org/multipage/#dom-element-insertadjacenthtml>
    pub(crate) fn fragment_parsing_context(
        owner_doc: &Document,
        element: Option<&Self>,
        can_gc: CanGc,
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
            _ => DomRoot::upcast(HTMLBodyElement::new(
                local_name!("body"),
                None,
                owner_doc,
                None,
                can_gc,
            )),
        }
    }

    // https://fullscreen.spec.whatwg.org/#fullscreen-element-ready-check
    pub(crate) fn fullscreen_element_ready_check(&self) -> bool {
        if !self.is_connected() {
            return false;
        }
        self.owner_document().get_allow_fullscreen()
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

    fn is_sequentially_focusable(&self) -> bool {
        let element = self.upcast::<Element>();
        let node = self.upcast::<Node>();
        if !node.is_connected() {
            return false;
        }

        if element.has_attribute(&local_name!("hidden")) {
            return false;
        }

        if self.disabled_state() {
            return false;
        }

        if element.has_attribute(&local_name!("tabindex")) {
            return true;
        }

        match node.type_id() {
            // <button>, <select>, <iframe>, and <textarea> are implicitly focusable.
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLSelectElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLIFrameElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLTextAreaElement,
            )) => true,

            // Links that generate actual links are focusable.
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) => element.has_attribute(&local_name!("href")),

            //TODO focusable if editing host
            //TODO focusable if "sorting interface th elements"
            _ => {
                // Draggable elements are focusable.
                element.get_string_attribute(&local_name!("draggable")) == "true"
            },
        }
    }

    pub(crate) fn update_sequentially_focusable_status(&self, can_gc: CanGc) {
        let node = self.upcast::<Node>();
        let is_sequentially_focusable = self.is_sequentially_focusable();
        node.set_flag(NodeFlags::SEQUENTIALLY_FOCUSABLE, is_sequentially_focusable);

        // https://html.spec.whatwg.org/multipage/#focus-fixup-rule
        if !is_sequentially_focusable {
            self.owner_document().perform_focus_fixup_rule(self, can_gc);
        }
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

    pub(crate) fn outer_html(&self, can_gc: CanGc) -> Fallible<DOMString> {
        match self.GetOuterHTML(can_gc)? {
            TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(str) => Ok(str),
            TrustedHTMLOrNullIsEmptyString::TrustedHTML(_) => unreachable!(),
        }
    }
}

impl ElementMethods<crate::DomTypeHolder> for Element {
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
        self.prefix.borrow().as_ref().map(|p| DOMString::from(&**p))
    }

    // https://dom.spec.whatwg.org/#dom-element-tagname
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

    // https://dom.spec.whatwg.org/#dom-element-id
    fn SetId(&self, id: DOMString, can_gc: CanGc) {
        self.set_atomic_attribute(&local_name!("id"), id, can_gc);
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn ClassName(&self) -> DOMString {
        self.get_string_attribute(&local_name!("class"))
    }

    // https://dom.spec.whatwg.org/#dom-element-classname
    fn SetClassName(&self, class: DOMString, can_gc: CanGc) {
        self.set_tokenlist_attribute(&local_name!("class"), class, can_gc);
    }

    // https://dom.spec.whatwg.org/#dom-element-classlist
    fn ClassList(&self, can_gc: CanGc) -> DomRoot<DOMTokenList> {
        self.class_list
            .or_init(|| DOMTokenList::new(self, &local_name!("class"), None, can_gc))
    }

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_getter!(Slot, "slot");

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_setter!(SetSlot, "slot");

    // https://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(&self, can_gc: CanGc) -> DomRoot<NamedNodeMap> {
        self.attr_list
            .or_init(|| NamedNodeMap::new(&self.owner_window(), self, can_gc))
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
        self.GetAttributeNode(name).map(|s| s.Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributens
    fn GetAttributeNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Option<DOMString> {
        self.GetAttributeNodeNS(namespace, local_name)
            .map(|attr| attr.Value())
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributenode
    fn GetAttributeNode(&self, name: DOMString) -> Option<DomRoot<Attr>> {
        self.get_attribute_by_name(name)
    }

    // https://dom.spec.whatwg.org/#dom-element-getattributenodens
    fn GetAttributeNodeNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
    ) -> Option<DomRoot<Attr>> {
        let namespace = &namespace_from_domstring(namespace);
        self.get_attribute(namespace, &LocalName::from(local_name))
    }

    // https://dom.spec.whatwg.org/#dom-element-toggleattribute
    fn ToggleAttribute(
        &self,
        name: DOMString,
        force: Option<bool>,
        can_gc: CanGc,
    ) -> Fallible<bool> {
        // Step 1.
        if !matches_name_production(&name) {
            return Err(Error::InvalidCharacter);
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
                        can_gc,
                    );
                    Ok(true)
                },
                // Step 4.2.
                Some(false) => Ok(false),
            },
            Some(_index) => match force {
                // Step 5.
                None | Some(false) => {
                    self.remove_attribute_by_name(&name, can_gc);
                    Ok(false)
                },
                // Step 6.
                Some(true) => Ok(true),
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-element-setattribute>
    fn SetAttribute(&self, name: DOMString, value: DOMString, can_gc: CanGc) -> ErrorResult {
        // Step 1. If qualifiedName does not match the Name production in XML,
        // then throw an "InvalidCharacterError" DOMException.
        if !matches_name_production(&name) {
            return Err(Error::InvalidCharacter);
        }

        // Step 2.
        let name = self.parsed_name(name);

        // Step 3-5.
        let value = self.parse_attribute(&ns!(), &name, value);
        self.set_first_matching_attribute(
            name.clone(),
            value,
            name.clone(),
            ns!(),
            None,
            |attr| *attr.name() == name,
            can_gc,
        );
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributens
    fn SetAttributeNS(
        &self,
        namespace: Option<DOMString>,
        qualified_name: DOMString,
        value: DOMString,
        can_gc: CanGc,
    ) -> ErrorResult {
        let (namespace, prefix, local_name) = validate_and_extract(namespace, &qualified_name)?;
        let qualified_name = LocalName::from(qualified_name);
        let value = self.parse_attribute(&namespace, &local_name, value);
        self.set_first_matching_attribute(
            local_name.clone(),
            value,
            qualified_name,
            namespace.clone(),
            prefix,
            |attr| *attr.local_name() == local_name && *attr.namespace() == namespace,
            can_gc,
        );
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributenode
    fn SetAttributeNode(&self, attr: &Attr, can_gc: CanGc) -> Fallible<Option<DomRoot<Attr>>> {
        // Step 1.
        if let Some(owner) = attr.GetOwnerElement() {
            if &*owner != self {
                return Err(Error::InUseAttribute);
            }
        }

        let vtable = vtable_for(self.upcast());

        // This ensures that the attribute is of the expected kind for this
        // specific element. This is inefficient and should probably be done
        // differently.
        attr.swap_value(&mut vtable.parse_plain_attribute(attr.local_name(), attr.Value()));

        // Step 2.
        let position = self.attrs.borrow().iter().position(|old_attr| {
            attr.namespace() == old_attr.namespace() && attr.local_name() == old_attr.local_name()
        });

        if let Some(position) = position {
            let old_attr = DomRoot::from_ref(&*self.attrs.borrow()[position]);

            // Step 3.
            if &*old_attr == attr {
                return Ok(Some(DomRoot::from_ref(attr)));
            }

            // Step 4.
            if self.is_custom() {
                let old_name = old_attr.local_name().clone();
                let old_value = DOMString::from(&**old_attr.value());
                let new_value = DOMString::from(&**attr.value());
                let namespace = old_attr.namespace().clone();
                let reaction = CallbackReaction::AttributeChanged(
                    old_name,
                    Some(old_value),
                    Some(new_value),
                    namespace,
                );
                ScriptThread::enqueue_callback_reaction(self, reaction, None);
            }
            self.will_mutate_attr(attr);
            attr.set_owner(Some(self));
            self.attrs.borrow_mut()[position] = Dom::from_ref(attr);
            old_attr.set_owner(None);
            if is_relevant_attribute(attr.namespace(), attr.local_name()) {
                vtable.attribute_mutated(
                    attr,
                    AttributeMutation::Set(Some(&old_attr.value())),
                    can_gc,
                );
            }

            // Step 6.
            Ok(Some(old_attr))
        } else {
            // Step 5.
            attr.set_owner(Some(self));
            self.push_attribute(attr, can_gc);

            // Step 6.
            Ok(None)
        }
    }

    // https://dom.spec.whatwg.org/#dom-element-setattributenodens
    fn SetAttributeNodeNS(&self, attr: &Attr, can_gc: CanGc) -> Fallible<Option<DomRoot<Attr>>> {
        self.SetAttributeNode(attr, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattribute
    fn RemoveAttribute(&self, name: DOMString, can_gc: CanGc) {
        let name = self.parsed_name(name);
        self.remove_attribute_by_name(&name, can_gc);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributens
    fn RemoveAttributeNS(
        &self,
        namespace: Option<DOMString>,
        local_name: DOMString,
        can_gc: CanGc,
    ) {
        let namespace = namespace_from_domstring(namespace);
        let local_name = LocalName::from(local_name);
        self.remove_attribute(&namespace, &local_name, can_gc);
    }

    // https://dom.spec.whatwg.org/#dom-element-removeattributenode
    fn RemoveAttributeNode(&self, attr: &Attr, can_gc: CanGc) -> Fallible<DomRoot<Attr>> {
        self.remove_first_matching_attribute(|a| a == attr, can_gc)
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
    fn GetElementsByTagName(&self, localname: DOMString, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_qualified_name(
            &window,
            self.upcast(),
            LocalName::from(&*localname),
            can_gc,
        )
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(
        &self,
        maybe_ns: Option<DOMString>,
        localname: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_tag_name_ns(&window, self.upcast(), localname, maybe_ns, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_class_name(&window, self.upcast(), classes, can_gc)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-getclientrects
    fn GetClientRects(&self, can_gc: CanGc) -> DomRoot<DOMRectList> {
        let win = self.owner_window();
        let raw_rects = self.upcast::<Node>().content_boxes(can_gc);
        let rects: Vec<DomRoot<DOMRect>> = raw_rects
            .iter()
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

    // https://drafts.csswg.org/cssom-view/#dom-element-getboundingclientrect
    fn GetBoundingClientRect(&self, can_gc: CanGc) -> DomRoot<DOMRect> {
        let win = self.owner_window();
        let rect = self.upcast::<Node>().bounding_content_box_or_zero(can_gc);
        DOMRect::new(
            win.upcast(),
            rect.origin.x.to_f64_px(),
            rect.origin.y.to_f64_px(),
            rect.size.width.to_f64_px(),
            rect.size.height.to_f64_px(),
            can_gc,
        )
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    fn Scroll(&self, options: &ScrollToOptions, can_gc: CanGc) {
        // Step 1
        let left = options.left.unwrap_or(self.ScrollLeft(can_gc));
        let top = options.top.unwrap_or(self.ScrollTop(can_gc));
        self.scroll(left, top, options.parent.behavior, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scroll
    fn Scroll_(&self, x: f64, y: f64, can_gc: CanGc) {
        self.scroll(x, y, ScrollBehavior::Auto, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollto
    fn ScrollTo(&self, options: &ScrollToOptions, can_gc: CanGc) {
        self.Scroll(options, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollto
    fn ScrollTo_(&self, x: f64, y: f64, can_gc: CanGc) {
        self.Scroll_(x, y, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollby
    fn ScrollBy(&self, options: &ScrollToOptions, can_gc: CanGc) {
        // Step 2
        let delta_left = options.left.unwrap_or(0.0f64);
        let delta_top = options.top.unwrap_or(0.0f64);
        let left = self.ScrollLeft(can_gc);
        let top = self.ScrollTop(can_gc);
        self.scroll(
            left + delta_left,
            top + delta_top,
            options.parent.behavior,
            can_gc,
        );
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollby
    fn ScrollBy_(&self, x: f64, y: f64, can_gc: CanGc) {
        let left = self.ScrollLeft(can_gc);
        let top = self.ScrollTop(can_gc);
        self.scroll(left + x, top + y, ScrollBehavior::Auto, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn ScrollTop(&self, can_gc: CanGc) -> f64 {
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
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body(can_gc)
        {
            return win.ScrollY() as f64;
        }

        // Step 8
        if !self.has_css_layout_box(can_gc) {
            return 0.0;
        }

        // Step 9
        let point = win.scroll_offset_query(node, can_gc);
        point.y.abs() as f64
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn SetScrollTop(&self, y_: f64, can_gc: CanGc) {
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
                win.scroll(win.ScrollX() as f64, y, behavior, can_gc);
            }

            return;
        }

        // Step 9
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body(can_gc)
        {
            win.scroll(win.ScrollX() as f64, y, behavior, can_gc);
            return;
        }

        // Step 10
        if !self.has_css_layout_box(can_gc) ||
            !self.has_scrolling_box(can_gc) ||
            !self.has_overflow(can_gc)
        {
            return;
        }

        // Step 11
        win.scroll_node(node, self.ScrollLeft(can_gc), y, behavior, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    fn ScrollLeft(&self, can_gc: CanGc) -> f64 {
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
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body(can_gc)
        {
            return win.ScrollX() as f64;
        }

        // Step 8
        if !self.has_css_layout_box(can_gc) {
            return 0.0;
        }

        // Step 9
        let point = win.scroll_offset_query(node, can_gc);
        point.x.abs() as f64
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollleft
    fn SetScrollLeft(&self, x_: f64, can_gc: CanGc) {
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

            win.scroll(x, win.ScrollY() as f64, behavior, can_gc);
            return;
        }

        // Step 9
        if doc.GetBody().as_deref() == self.downcast::<HTMLElement>() &&
            doc.quirks_mode() == QuirksMode::Quirks &&
            !self.is_potentially_scrollable_body(can_gc)
        {
            win.scroll(x, win.ScrollY() as f64, behavior, can_gc);
            return;
        }

        // Step 10
        if !self.has_css_layout_box(can_gc) ||
            !self.has_scrolling_box(can_gc) ||
            !self.has_overflow(can_gc)
        {
            return;
        }

        // Step 11
        win.scroll_node(node, x, self.ScrollTop(can_gc), behavior, can_gc);
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth
    fn ScrollWidth(&self, can_gc: CanGc) -> i32 {
        self.upcast::<Node>().scroll_area(can_gc).size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollheight
    fn ScrollHeight(&self, can_gc: CanGc) -> i32 {
        self.upcast::<Node>().scroll_area(can_gc).size.height
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clienttop
    fn ClientTop(&self, can_gc: CanGc) -> i32 {
        self.client_rect(can_gc).origin.y
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientleft
    fn ClientLeft(&self, can_gc: CanGc) -> i32 {
        self.client_rect(can_gc).origin.x
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientwidth
    fn ClientWidth(&self, can_gc: CanGc) -> i32 {
        self.client_rect(can_gc).size.width
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-clientheight
    fn ClientHeight(&self, can_gc: CanGc) -> i32 {
        self.client_rect(can_gc).size.height
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-sethtmlunsafe>
    fn SetHTMLUnsafe(&self, html: TrustedHTMLOrString, can_gc: CanGc) -> ErrorResult {
        // Step 1. Let compliantHTML be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, html, "Element setHTMLUnsafe", and "script".
        let html = TrustedHTML::get_trusted_script_compliant_string(
            &self.owner_global(),
            html,
            "Element",
            "setHTMLUnsafe",
            can_gc,
        )?;
        // Step 2. Let target be this's template contents if this is a template element; otherwise this.
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            DomRoot::upcast(template.Content(can_gc))
        } else {
            DomRoot::from_ref(self.upcast())
        };

        // Step 3. Unsafely set HTML given target, this, and compliantHTML
        Node::unsafely_set_html(&target, self, html, can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-gethtml>
    fn GetHTML(&self, options: &GetHTMLOptions, can_gc: CanGc) -> DOMString {
        // > Element's getHTML(options) method steps are to return the result of HTML fragment serialization
        // > algorithm with this, options["serializableShadowRoots"], and options["shadowRoots"].
        self.upcast::<Node>().html_serialize(
            TraversalScope::ChildrenOnly(None),
            options.serializableShadowRoots,
            options.shadowRoots.clone(),
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn GetInnerHTML(&self, can_gc: CanGc) -> Fallible<TrustedHTMLOrNullIsEmptyString> {
        let qname = QualName::new(
            self.prefix().clone(),
            self.namespace().clone(),
            self.local_name().clone(),
        );

        // FIXME: This should use the fragment serialization algorithm, which takes
        // care of distinguishing between html/xml documents
        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>()
                .html_serialize(ChildrenOnly(Some(qname)), false, vec![], can_gc)
        } else {
            self.upcast::<Node>()
                .xml_serialize(XmlChildrenOnly(Some(qname)))
        };

        Ok(TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(result))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn SetInnerHTML(&self, value: TrustedHTMLOrNullIsEmptyString, can_gc: CanGc) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, the given value, "Element innerHTML", and "script".
        let value = TrustedHTML::get_trusted_script_compliant_string(
            &self.owner_global(),
            value.convert(),
            "Element",
            "innerHTML",
            can_gc,
        )?;
        // https://github.com/w3c/DOM-Parsing/issues/1
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            // Step 4: If context is a template element, then set context to
            // the template element's template contents (a DocumentFragment).
            DomRoot::upcast(template.Content(can_gc))
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
            Node::SetTextContent(&target, Some(value), can_gc);
            return Ok(());
        }

        // Step 3: Let fragment be the result of invoking the fragment parsing algorithm steps
        // with context and compliantString.
        let frag = self.parse_fragment(value, can_gc)?;

        // Step 5: Replace all with fragment within context.
        Node::replace_all(Some(frag.upcast()), &target, can_gc);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn GetOuterHTML(&self, can_gc: CanGc) -> Fallible<TrustedHTMLOrNullIsEmptyString> {
        // FIXME: This should use the fragment serialization algorithm, which takes
        // care of distinguishing between html/xml documents
        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>()
                .html_serialize(IncludeNode, false, vec![], can_gc)
        } else {
            self.upcast::<Node>().xml_serialize(XmlIncludeNode)
        };

        Ok(TrustedHTMLOrNullIsEmptyString::NullIsEmptyString(result))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn SetOuterHTML(&self, value: TrustedHTMLOrNullIsEmptyString, can_gc: CanGc) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, the given value, "Element outerHTML", and "script".
        let value = TrustedHTML::get_trusted_script_compliant_string(
            &self.owner_global(),
            value.convert(),
            "Element",
            "outerHTML",
            can_gc,
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
            NodeTypeId::Document(_) => return Err(Error::NoModificationAllowed),

            // Step 5: If parent is a DocumentFragment, set parent to the result of
            // creating an element given this's node document, "body", and the HTML namespace.
            NodeTypeId::DocumentFragment(_) => {
                let body_elem = Element::create(
                    QualName::new(None, ns!(html), local_name!("body")),
                    None,
                    &context_document,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Synchronous,
                    None,
                    can_gc,
                );
                DomRoot::upcast(body_elem)
            },
            _ => context_node.GetParentElement().unwrap(),
        };

        // Step 6: Let fragment be the result of invoking the
        // fragment parsing algorithm steps given parent and compliantString.
        let frag = parent.parse_fragment(value, can_gc)?;
        // Step 7: Replace this with fragment within this's parent.
        context_parent.ReplaceChild(frag.upcast(), context_node, can_gc)?;
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    fn GetPreviousElementSibling(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .preceding_siblings()
            .filter_map(DomRoot::downcast)
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    fn GetNextElementSibling(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .following_siblings()
            .filter_map(DomRoot::downcast)
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-children
    fn Children(&self, can_gc: CanGc) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::children(&window, self.upcast(), can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    fn GetFirstElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().child_elements().next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    fn GetLastElementChild(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>()
            .rev_children()
            .filter_map(DomRoot::downcast::<Element>)
            .next()
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    fn ChildElementCount(&self) -> u32 {
        self.upcast::<Node>().child_elements().count() as u32
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn Prepend(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().prepend(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn Append(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().append(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
    fn ReplaceChildren(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_children(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn QuerySelector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let root = self.upcast::<Node>();
        root.query_selector(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    fn QuerySelectorAll(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let root = self.upcast::<Node>();
        root.query_selector_all(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn Before(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().before(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn After(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().after(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn ReplaceWith(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        self.upcast::<Node>().replace_with(nodes, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-childnode-remove
    fn Remove(&self, can_gc: CanGc) {
        self.upcast::<Node>().remove_self(can_gc);
    }

    // https://dom.spec.whatwg.org/#dom-element-matches
    fn Matches(&self, selectors: DOMString) -> Fallible<bool> {
        let doc = self.owner_document();
        let url = doc.url();
        let selectors = match SelectorParser::parse_author_origin_no_namespace(
            &selectors,
            &UrlExtraData(url.get_arc()),
        ) {
            Err(_) => return Err(Error::Syntax),
            Ok(selectors) => selectors,
        };

        let quirks_mode = doc.quirks_mode();
        let element = DomRoot::from_ref(self);

        Ok(dom_apis::element_matches(
            &SelectorWrapper::Borrowed(&element),
            &selectors,
            quirks_mode,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-element-webkitmatchesselector
    fn WebkitMatchesSelector(&self, selectors: DOMString) -> Fallible<bool> {
        self.Matches(selectors)
    }

    // https://dom.spec.whatwg.org/#dom-element-closest
    fn Closest(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        let doc = self.owner_document();
        let url = doc.url();
        let selectors = match SelectorParser::parse_author_origin_no_namespace(
            &selectors,
            &UrlExtraData(url.get_arc()),
        ) {
            Err(_) => return Err(Error::Syntax),
            Ok(selectors) => selectors,
        };

        let quirks_mode = doc.quirks_mode();
        Ok(dom_apis::element_closest(
            SelectorWrapper::Owned(DomRoot::from_ref(self)),
            &selectors,
            quirks_mode,
        )
        .map(SelectorWrapper::into_owned))
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacentelement
    fn InsertAdjacentElement(
        &self,
        where_: DOMString,
        element: &Element,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<Element>>> {
        let where_ = where_.parse::<AdjacentPosition>()?;
        let inserted_node = self.insert_adjacent(where_, element.upcast(), can_gc)?;
        Ok(inserted_node.map(|node| DomRoot::downcast(node).unwrap()))
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacenttext
    fn InsertAdjacentText(&self, where_: DOMString, data: DOMString, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let text = Text::new(data, &self.owner_document(), can_gc);

        // Step 2.
        let where_ = where_.parse::<AdjacentPosition>()?;
        self.insert_adjacent(where_, text.upcast(), can_gc)
            .map(|_| ())
    }

    // https://w3c.github.io/DOM-Parsing/#dom-element-insertadjacenthtml
    fn InsertAdjacentHTML(
        &self,
        position: DOMString,
        text: TrustedHTMLOrString,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 1: Let compliantString be the result of invoking the
        // Get Trusted Type compliant string algorithm with TrustedHTML,
        // this's relevant global object, string, "Element insertAdjacentHTML", and "script".
        let text = TrustedHTML::get_trusted_script_compliant_string(
            &self.owner_global(),
            text,
            "Element",
            "insertAdjacentHTML",
            can_gc,
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
                        return Err(Error::NoModificationAllowed);
                    },
                    None => return Err(Error::NoModificationAllowed),
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
            &context.owner_doc(),
            context.downcast::<Element>(),
            can_gc,
        );

        // Step 5: Let fragment be the result of invoking the
        // fragment parsing algorithm steps with context and compliantString.
        let fragment = context.parse_fragment(text, can_gc)?;

        // Step 6.
        self.insert_adjacent(position, fragment.upcast(), can_gc)
            .map(|_| ())
    }

    // check-tidy: no specs after this line
    fn EnterFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.enter_formal_activation_state();
                Ok(())
            },
            None => Err(Error::NotSupported),
        }
    }

    fn ExitFormalActivationState(&self) -> ErrorResult {
        match self.as_maybe_activatable() {
            Some(a) => {
                a.exit_formal_activation_state();
                Ok(())
            },
            None => Err(Error::NotSupported),
        }
    }

    // https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen
    fn RequestFullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        let doc = self.owner_document();
        doc.enter_fullscreen(self, can_gc)
    }

    // https://dom.spec.whatwg.org/#dom-element-attachshadow
    fn AttachShadow(&self, init: &ShadowRootInit, can_gc: CanGc) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1. Run attach a shadow root with this, init["mode"], init["clonable"], init["serializable"],
        // init["delegatesFocus"], and init["slotAssignment"].
        let shadow_root = self.attach_shadow(
            IsUserAgentWidget::No,
            init.mode,
            init.clonable,
            init.serializable,
            init.delegatesFocus,
            init.slotAssignment,
            can_gc,
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

    fn GetRole(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("role"))
    }

    fn SetRole(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("role"), value, can_gc);
    }

    fn GetAriaAtomic(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-atomic"))
    }

    fn SetAriaAtomic(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-atomic"), value, can_gc);
    }

    fn GetAriaAutoComplete(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-autocomplete"))
    }

    fn SetAriaAutoComplete(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-autocomplete"), value, can_gc);
    }

    fn GetAriaBrailleLabel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-braillelabel"))
    }

    fn SetAriaBrailleLabel(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-braillelabel"), value, can_gc);
    }

    fn GetAriaBrailleRoleDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-brailleroledescription"))
    }

    fn SetAriaBrailleRoleDescription(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(
            &local_name!("aria-brailleroledescription"),
            value,
            can_gc,
        );
    }

    fn GetAriaBusy(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-busy"))
    }

    fn SetAriaBusy(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-busy"), value, can_gc);
    }

    fn GetAriaChecked(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-checked"))
    }

    fn SetAriaChecked(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-checked"), value, can_gc);
    }

    fn GetAriaColCount(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colcount"))
    }

    fn SetAriaColCount(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-colcount"), value, can_gc);
    }

    fn GetAriaColIndex(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colindex"))
    }

    fn SetAriaColIndex(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-colindex"), value, can_gc);
    }

    fn GetAriaColIndexText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colindextext"))
    }

    fn SetAriaColIndexText(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-colindextext"), value, can_gc);
    }

    fn GetAriaColSpan(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-colspan"))
    }

    fn SetAriaColSpan(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-colspan"), value, can_gc);
    }

    fn GetAriaCurrent(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-current"))
    }

    fn SetAriaCurrent(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-current"), value, can_gc);
    }

    fn GetAriaDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-description"))
    }

    fn SetAriaDescription(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-description"), value, can_gc);
    }

    fn GetAriaDisabled(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-disabled"))
    }

    fn SetAriaDisabled(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-disabled"), value, can_gc);
    }

    fn GetAriaExpanded(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-expanded"))
    }

    fn SetAriaExpanded(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-expanded"), value, can_gc);
    }

    fn GetAriaHasPopup(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-haspopup"))
    }

    fn SetAriaHasPopup(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-haspopup"), value, can_gc);
    }

    fn GetAriaHidden(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-hidden"))
    }

    fn SetAriaHidden(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-hidden"), value, can_gc);
    }

    fn GetAriaInvalid(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-invalid"))
    }

    fn SetAriaInvalid(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-invalid"), value, can_gc);
    }

    fn GetAriaKeyShortcuts(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-keyshortcuts"))
    }

    fn SetAriaKeyShortcuts(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-keyshortcuts"), value, can_gc);
    }

    fn GetAriaLabel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-label"))
    }

    fn SetAriaLabel(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-label"), value, can_gc);
    }

    fn GetAriaLevel(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-level"))
    }

    fn SetAriaLevel(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-level"), value, can_gc);
    }

    fn GetAriaLive(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-live"))
    }

    fn SetAriaLive(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-live"), value, can_gc);
    }

    fn GetAriaModal(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-modal"))
    }

    fn SetAriaModal(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-modal"), value, can_gc);
    }

    fn GetAriaMultiLine(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-multiline"))
    }

    fn SetAriaMultiLine(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-multiline"), value, can_gc);
    }

    fn GetAriaMultiSelectable(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-multiselectable"))
    }

    fn SetAriaMultiSelectable(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-multiselectable"), value, can_gc);
    }

    fn GetAriaOrientation(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-orientation"))
    }

    fn SetAriaOrientation(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-orientation"), value, can_gc);
    }

    fn GetAriaPlaceholder(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-placeholder"))
    }

    fn SetAriaPlaceholder(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-placeholder"), value, can_gc);
    }

    fn GetAriaPosInSet(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-posinset"))
    }

    fn SetAriaPosInSet(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-posinset"), value, can_gc);
    }

    fn GetAriaPressed(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-pressed"))
    }

    fn SetAriaPressed(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-pressed"), value, can_gc);
    }

    fn GetAriaReadOnly(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-readonly"))
    }

    fn SetAriaReadOnly(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-readonly"), value, can_gc);
    }

    fn GetAriaRelevant(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-relevant"))
    }

    fn SetAriaRelevant(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-relevant"), value, can_gc);
    }

    fn GetAriaRequired(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-required"))
    }

    fn SetAriaRequired(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-required"), value, can_gc);
    }

    fn GetAriaRoleDescription(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-roledescription"))
    }

    fn SetAriaRoleDescription(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-roledescription"), value, can_gc);
    }

    fn GetAriaRowCount(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowcount"))
    }

    fn SetAriaRowCount(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-rowcount"), value, can_gc);
    }

    fn GetAriaRowIndex(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowindex"))
    }

    fn SetAriaRowIndex(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-rowindex"), value, can_gc);
    }

    fn GetAriaRowIndexText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowindextext"))
    }

    fn SetAriaRowIndexText(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-rowindextext"), value, can_gc);
    }

    fn GetAriaRowSpan(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-rowspan"))
    }

    fn SetAriaRowSpan(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-rowspan"), value, can_gc);
    }

    fn GetAriaSelected(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-selected"))
    }

    fn SetAriaSelected(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-selected"), value, can_gc);
    }

    fn GetAriaSetSize(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-setsize"))
    }

    fn SetAriaSetSize(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-setsize"), value, can_gc);
    }

    fn GetAriaSort(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-sort"))
    }

    fn SetAriaSort(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-sort"), value, can_gc);
    }

    fn GetAriaValueMax(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuemax"))
    }

    fn SetAriaValueMax(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-valuemax"), value, can_gc);
    }

    fn GetAriaValueMin(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuemin"))
    }

    fn SetAriaValueMin(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-valuemin"), value, can_gc);
    }

    fn GetAriaValueNow(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuenow"))
    }

    fn SetAriaValueNow(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-valuenow"), value, can_gc);
    }

    fn GetAriaValueText(&self) -> Option<DOMString> {
        self.get_nullable_string_attribute(&local_name!("aria-valuetext"))
    }

    fn SetAriaValueText(&self, value: Option<DOMString>, can_gc: CanGc) {
        self.set_nullable_string_attribute(&local_name!("aria-valuetext"), value, can_gc);
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
        self.ensure_rare_data()
            .part
            .or_init(|| DOMTokenList::new(self, &local_name!("part"), None, CanGc::note()))
    }
}

impl VirtualMethods for Element {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Node>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        // FIXME: This should be more fine-grained, not all elements care about these.
        if attr.local_name() == &local_name!("width") ||
            attr.local_name() == &local_name!("height") ||
            attr.local_name() == &local_name!("lang")
        {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        let node = self.upcast::<Node>();
        let doc = node.owner_doc();
        match attr.local_name() {
            &local_name!("tabindex") | &local_name!("draggable") | &local_name!("hidden") => {
                self.update_sequentially_focusable_status(can_gc)
            },
            &local_name!("style") => self.update_style_attribute(attr, mutation),
            &local_name!("id") => {
                *self.id_attribute.borrow_mut() = mutation.new_value(attr).and_then(|value| {
                    let value = value.as_atom();
                    if value != &atom!("") {
                        Some(value.clone())
                    } else {
                        None
                    }
                });

                let containing_shadow_root = self.containing_shadow_root();
                if node.is_in_a_document_tree() || node.is_in_a_shadow_tree() {
                    let value = attr.value().as_atom().clone();
                    match mutation {
                        AttributeMutation::Set(old_value) => {
                            if let Some(old_value) = old_value {
                                let old_value = old_value.as_atom().clone();
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.unregister_element_id(self, old_value, can_gc);
                                } else {
                                    doc.unregister_element_id(self, old_value, can_gc);
                                }
                            }
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.register_element_id(self, value, can_gc);
                                } else {
                                    doc.register_element_id(self, value, can_gc);
                                }
                            }
                        },
                        AttributeMutation::Removed => {
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.unregister_element_id(self, value, can_gc);
                                } else {
                                    doc.unregister_element_id(self, value, can_gc);
                                }
                            }
                        },
                    }
                }
            },
            &local_name!("name") => {
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
                        AttributeMutation::Set(old_value) => {
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
            &local_name!("slot") => {
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

        // Make sure we rev the version even if we didn't dirty the node. If we
        // don't do this, various attribute-dependent htmlcollections (like those
        // generated by getElementsByClassName) might become stale.
        node.rev_version();
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("id") => AttrValue::from_atomic(value.into()),
            local_name!("name") => AttrValue::from_atomic(value.into()),
            local_name!("class") | local_name!("part") => {
                AttrValue::from_serialized_tokenlist(value.into())
            },
            local_name!("exportparts") => AttrValue::from_shadow_parts(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        if let Some(f) = self.as_maybe_form_control() {
            f.bind_form_control_to_tree(can_gc);
        }

        let doc = self.owner_document();

        if let Some(ref shadow_root) = self.shadow_root() {
            shadow_root.bind_to_tree(context, can_gc);
        }

        if !context.is_in_tree() {
            return;
        }

        self.update_sequentially_focusable_status(can_gc);

        if let Some(ref id) = *self.id_attribute.borrow() {
            if let Some(shadow_root) = self.containing_shadow_root() {
                shadow_root.register_element_id(self, id.clone(), can_gc);
            } else {
                doc.register_element_id(self, id.clone(), can_gc);
            }
        }
        if let Some(ref name) = self.name_attribute() {
            if self.containing_shadow_root().is_none() {
                doc.register_element_name(self, name.clone());
            }
        }

        // This is used for layout optimization.
        doc.increment_dom_count();
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

        self.update_sequentially_focusable_status(can_gc);

        let doc = self.owner_document();

        let fullscreen = doc.GetFullscreenElement();
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
        // This is used for layout optimization.
        doc.decrement_dom_count();
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }

        let flags = self.selector_flags.get();
        if flags.intersects(ElementSelectorFlags::HAS_SLOW_SELECTOR) {
            // All children of this node need to be restyled when any child changes.
            self.upcast::<Node>().dirty(NodeDamage::Other);
        } else {
            if flags.intersects(ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
                if let Some(next_child) = mutation.next_child() {
                    for child in next_child.inclusively_following_siblings() {
                        if child.is::<Element>() {
                            child.dirty(NodeDamage::Other);
                        }
                    }
                }
            }
            if flags.intersects(ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR) {
                if let Some(child) = mutation.modified_edge_element() {
                    child.dirty(NodeDamage::Other);
                }
            }
        }
    }

    fn adopting_steps(&self, old_doc: &Document, can_gc: CanGc) {
        self.super_type().unwrap().adopting_steps(old_doc, can_gc);

        if self.owner_document().is_html_document() != old_doc.is_html_document() {
            self.tag_name.clear();
        }
    }

    fn post_connection_steps(&self) {
        if let Some(s) = self.super_type() {
            s.post_connection_steps();
        }

        self.update_nonce_post_connection();
    }

    /// <https://html.spec.whatwg.org/multipage/#nonce-attributes%3Aconcept-node-clone-ext>
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        can_gc: CanGc,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children, can_gc);
        }
        let elem = copy.downcast::<Element>().unwrap();
        if let Some(rare_data) = self.rare_data().as_ref() {
            elem.update_nonce_internal_slot(rare_data.cryptographic_nonce.clone());
        }
    }
}

#[derive(Clone, PartialEq)]
/// A type that wraps a DomRoot value so we can implement the SelectorsElement
/// trait without violating the orphan rule. Since the trait assumes that the
/// return type and self type of various methods is the same type that it is
/// implemented against, we need to be able to represent multiple ownership styles.
pub enum SelectorWrapper<'a> {
    Borrowed(&'a DomRoot<Element>),
    Owned(DomRoot<Element>),
}

impl fmt::Debug for SelectorWrapper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl Deref for SelectorWrapper<'_> {
    type Target = DomRoot<Element>;

    fn deref(&self) -> &Self::Target {
        match self {
            SelectorWrapper::Owned(r) => r,
            SelectorWrapper::Borrowed(r) => r,
        }
    }
}

impl SelectorWrapper<'_> {
    fn into_owned(self) -> DomRoot<Element> {
        match self {
            SelectorWrapper::Owned(r) => r,
            SelectorWrapper::Borrowed(r) => r.clone(),
        }
    }
}

impl SelectorsElement for SelectorWrapper<'_> {
    type Impl = SelectorImpl;

    #[allow(unsafe_code)]
    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*self.reflector().get_jsobject().get() })
    }

    fn parent_element(&self) -> Option<Self> {
        self.upcast::<Node>()
            .GetParentElement()
            .map(SelectorWrapper::Owned)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        match self.upcast::<Node>().GetParentNode() {
            None => false,
            Some(node) => node.is::<ShadowRoot>(),
        }
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        self.containing_shadow_root()
            .map(|shadow_root| shadow_root.Host())
            .map(SelectorWrapper::Owned)
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pseudo: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.node
            .preceding_siblings()
            .filter_map(DomRoot::downcast)
            .next()
            .map(SelectorWrapper::Owned)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.node
            .following_siblings()
            .filter_map(DomRoot::downcast)
            .next()
            .map(SelectorWrapper::Owned)
    }

    fn first_element_child(&self) -> Option<Self> {
        self.GetFirstElementChild().map(SelectorWrapper::Owned)
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        operation: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ns) => self
                .get_attribute(ns, local_name)
                .is_some_and(|attr| attr.value().eval_selector(operation)),
            NamespaceConstraint::Any => self.attrs.borrow().iter().any(|attr| {
                *attr.local_name() == **local_name && attr.value().eval_selector(operation)
            }),
        }
    }

    fn is_root(&self) -> bool {
        Element::is_root(self)
    }

    fn is_empty(&self) -> bool {
        self.node.children().all(|node| {
            !node.is::<Element>() &&
                match node.downcast::<Text>() {
                    None => true,
                    Some(text) => text.upcast::<CharacterData>().data().is_empty(),
                }
        })
    }

    fn has_local_name(&self, local_name: &LocalName) -> bool {
        Element::local_name(self) == local_name
    }

    fn has_namespace(&self, ns: &Namespace) -> bool {
        Element::namespace(self) == ns
    }

    fn is_same_type(&self, other: &Self) -> bool {
        Element::local_name(self) == Element::local_name(other) &&
            Element::namespace(self) == Element::namespace(other)
    }

    fn match_non_ts_pseudo_class(
        &self,
        pseudo_class: &NonTSPseudoClass,
        _: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        match *pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link | NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::ServoNonZeroBorder => match self.downcast::<HTMLTableElement>() {
                None => false,
                Some(this) => match this.get_border() {
                    None | Some(0) => false,
                    Some(_) => true,
                },
            },

            NonTSPseudoClass::CustomState(ref state) => self.has_custom_state(&state.0),

            // FIXME(heycam): This is wrong, since extended_filtering accepts
            // a string containing commas (separating each language tag in
            // a list) but the pseudo-class instead should be parsing and
            // storing separate <ident> or <string>s for each language tag.
            NonTSPseudoClass::Lang(ref lang) => {
                extended_filtering(&self.upcast::<Node>().get_lang().unwrap_or_default(), lang)
            },

            NonTSPseudoClass::ReadOnly => {
                !Element::state(self).contains(NonTSPseudoClass::ReadWrite.state_flag())
            },

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Autofill |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Default |
            NonTSPseudoClass::Defined |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::FocusVisible |
            NonTSPseudoClass::FocusWithin |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::InRange |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::Invalid |
            NonTSPseudoClass::Modal |
            NonTSPseudoClass::MozMeterOptimum |
            NonTSPseudoClass::MozMeterSubOptimum |
            NonTSPseudoClass::MozMeterSubSubOptimum |
            NonTSPseudoClass::Optional |
            NonTSPseudoClass::OutOfRange |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::PopoverOpen |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::Required |
            NonTSPseudoClass::Target |
            NonTSPseudoClass::UserInvalid |
            NonTSPseudoClass::UserValid |
            NonTSPseudoClass::Valid => Element::state(self).contains(pseudo_class.state_flag()),
        }
    }

    fn is_link(&self) -> bool {
        // FIXME: This is HTML only.
        let node = self.upcast::<Node>();
        match node.type_id() {
            // https://html.spec.whatwg.org/multipage/#selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
                self.has_attribute(&local_name!("href"))
            },
            _ => false,
        }
    }

    fn has_id(&self, id: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        self.id_attribute
            .borrow()
            .as_ref()
            .is_some_and(|atom| case_sensitivity.eq_atom(id, atom))
    }

    fn is_part(&self, name: &AtomIdent) -> bool {
        Element::is_part(self, name, CaseSensitivity::CaseSensitive)
    }

    fn imported_part(&self, _: &AtomIdent) -> Option<AtomIdent> {
        None
    }

    fn has_class(&self, name: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        Element::has_class(self, name, case_sensitivity)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.html_element_in_html_document()
    }

    fn is_html_slot_element(&self) -> bool {
        self.is_html_element() && self.local_name() == &local_name!("slot")
    }

    fn apply_selector_flags(&self, flags: ElementSelectorFlags) {
        // Handle flags that apply to the element.
        let self_flags = flags.for_self();
        if !self_flags.is_empty() {
            #[allow(unsafe_code)]
            unsafe {
                Dom::from_ref(&***self)
                    .to_layout()
                    .insert_selector_flags(self_flags);
            }
        }

        // Handle flags that apply to the parent.
        let parent_flags = flags.for_parent();
        if !parent_flags.is_empty() {
            if let Some(p) = self.parent_element() {
                #[allow(unsafe_code)]
                unsafe {
                    Dom::from_ref(&**p)
                        .to_layout()
                        .insert_selector_flags(parent_flags);
                }
            }
        }
    }

    fn add_element_unique_hashes(&self, filter: &mut BloomFilter) -> bool {
        let mut f = |hash| filter.insert_hash(hash & BLOOM_HASH_MASK);

        // We can't use style::bloom::each_relevant_element_hash(*self, f)
        // since DomRoot<Element> doesn't have the TElement trait.
        f(Element::local_name(self).get_hash());
        f(Element::namespace(self).get_hash());

        if let Some(ref id) = *self.id_attribute.borrow() {
            f(id.get_hash());
        }

        if let Some(attr) = self.get_attribute(&ns!(), &local_name!("class")) {
            for class in attr.value().as_tokens() {
                f(AtomIdent::cast(class).get_hash());
            }
        }

        for attr in self.attrs.borrow().iter() {
            let name = style::values::GenericAtomIdent::cast(attr.local_name());
            if !style::bloom::is_attr_name_excluded_from_filter(name) {
                f(name.get_hash());
            }
        }

        true
    }

    fn has_custom_state(&self, _name: &AtomIdent) -> bool {
        false
    }
}

impl Element {
    fn client_rect(&self, can_gc: CanGc) -> Rect<i32> {
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

        let mut rect = self.upcast::<Node>().client_rect(can_gc);
        let in_quirks_mode = doc.quirks_mode() == QuirksMode::Quirks;

        if (in_quirks_mode && doc.GetBody().as_deref() == self.downcast::<HTMLElement>()) ||
            (!in_quirks_mode && *self.root_element() == *self)
        {
            let viewport_dimensions = doc.window().viewport_details().size.round().to_i32();
            rect.size = Size2D::<i32>::new(viewport_dimensions.width, viewport_dimensions.height);
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
        let element = match self.upcast::<Node>().type_id() {
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
        };
        element
    }

    pub(crate) fn is_invalid(&self, needs_update: bool, can_gc: CanGc) -> bool {
        if let Some(validatable) = self.as_maybe_validatable() {
            if needs_update {
                validatable
                    .validity_state()
                    .perform_validation_and_update(ValidationFlags::all(), can_gc);
            }
            return validatable.is_instance_validatable() && !validatable.satisfies_constraints();
        }

        if let Some(internals) = self.get_element_internals() {
            return internals.is_invalid();
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

    // https://html.spec.whatwg.org/multipage/#nearest-activatable-element
    pub(crate) fn nearest_activable_element(&self) -> Option<DomRoot<Element>> {
        match self.as_maybe_activatable() {
            Some(el) => Some(DomRoot::from_ref(el.as_element())),
            None => {
                let node = self.upcast::<Node>();
                for node in node.ancestors() {
                    if let Some(node) = node.downcast::<Element>() {
                        if node.as_maybe_activatable().is_some() {
                            return Some(DomRoot::from_ref(node));
                        }
                    }
                }
                None
            },
        }
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

        let node = self.upcast::<Node>();
        node.owner_doc().element_state_will_change(self);
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
        self.upcast::<Node>().dirty(NodeDamage::Other);
    }

    pub(crate) fn hover_state(&self) -> bool {
        self.state.get().contains(ElementState::HOVER)
    }

    pub(crate) fn set_hover_state(&self, value: bool) {
        self.set_state(ElementState::HOVER, value)
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

    pub(crate) fn placeholder_shown_state(&self) -> bool {
        self.state.get().contains(ElementState::PLACEHOLDER_SHOWN)
    }

    pub(crate) fn set_placeholder_shown_state(&self, value: bool) {
        if self.placeholder_shown_state() != value {
            self.set_state(ElementState::PLACEHOLDER_SHOWN, value);
            self.upcast::<Node>().dirty(NodeDamage::Other);
        }
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

#[derive(Clone, Copy)]
pub(crate) enum AttributeMutation<'a> {
    /// The attribute is set, keep track of old value.
    /// <https://dom.spec.whatwg.org/#attribute-is-set>
    Set(Option<&'a AttrValue>),

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
            AttributeMutation::Set(_) => Some(attr.value()),
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

pub(crate) struct ElementPerformFullscreenEnter {
    element: Trusted<Element>,
    promise: TrustedPromise,
    error: bool,
}

impl ElementPerformFullscreenEnter {
    pub(crate) fn new(
        element: Trusted<Element>,
        promise: TrustedPromise,
        error: bool,
    ) -> Box<ElementPerformFullscreenEnter> {
        Box::new(ElementPerformFullscreenEnter {
            element,
            promise,
            error,
        })
    }
}

impl TaskOnce for ElementPerformFullscreenEnter {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn run_once(self) {
        let element = self.element.root();
        let promise = self.promise.root();
        let document = element.owner_document();

        // Step 7.1
        if self.error || !element.fullscreen_element_ready_check() {
            document
                .upcast::<EventTarget>()
                .fire_event(atom!("fullscreenerror"), CanGc::note());
            promise.reject_error(
                Error::Type(String::from("fullscreen is not connected")),
                CanGc::note(),
            );
            return;
        }

        // TODO Step 7.2-4
        // Step 7.5
        element.set_fullscreen_state(true);
        document.set_fullscreen_element(Some(&element));

        // Step 7.6
        document
            .upcast::<EventTarget>()
            .fire_event(atom!("fullscreenchange"), CanGc::note());

        // Step 7.7
        promise.resolve_native(&(), CanGc::note());
    }
}

pub(crate) struct ElementPerformFullscreenExit {
    element: Trusted<Element>,
    promise: TrustedPromise,
}

impl ElementPerformFullscreenExit {
    pub(crate) fn new(
        element: Trusted<Element>,
        promise: TrustedPromise,
    ) -> Box<ElementPerformFullscreenExit> {
        Box::new(ElementPerformFullscreenExit { element, promise })
    }
}

impl TaskOnce for ElementPerformFullscreenExit {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn run_once(self) {
        let element = self.element.root();
        let document = element.owner_document();
        // TODO Step 9.1-5
        // Step 9.6
        element.set_fullscreen_state(false);
        document.set_fullscreen_element(None);

        // Step 9.8
        document
            .upcast::<EventTarget>()
            .fire_event(atom!("fullscreenchange"), CanGc::note());

        // Step 9.10
        self.promise.root().resolve_native(&(), CanGc::note());
    }
}

pub(crate) fn reflect_cross_origin_attribute(element: &Element) -> Option<DOMString> {
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

pub(crate) fn set_cross_origin_attribute(
    element: &Element,
    value: Option<DOMString>,
    can_gc: CanGc,
) {
    match value {
        Some(val) => element.set_string_attribute(&local_name!("crossorigin"), val, can_gc),
        None => {
            element.remove_attribute(&ns!(), &local_name!("crossorigin"), can_gc);
        },
    }
}

pub(crate) fn reflect_referrer_policy_attribute(element: &Element) -> DOMString {
    let attr =
        element.get_attribute_by_name(DOMString::from_string(String::from("referrerpolicy")));

    if let Some(mut val) = attr.map(|v| v.Value()) {
        val.make_ascii_lowercase();
        if val == "no-referrer" ||
            val == "no-referrer-when-downgrade" ||
            val == "same-origin" ||
            val == "origin" ||
            val == "strict-origin" ||
            val == "origin-when-cross-origin" ||
            val == "strict-origin-when-cross-origin" ||
            val == "unsafe-url"
        {
            return val;
        }
    }
    DOMString::new()
}

pub(crate) fn referrer_policy_for_element(element: &Element) -> ReferrerPolicy {
    element
        .get_attribute_by_name(DOMString::from_string(String::from("referrerpolicy")))
        .map(|attribute: DomRoot<Attr>| determine_policy_for_token(&attribute.Value()))
        .unwrap_or(element.owner_document().get_referrer_policy())
}

pub(crate) fn cors_setting_for_element(element: &Element) -> Option<CorsSettings> {
    reflect_cross_origin_attribute(element).and_then(|attr| match &*attr {
        "anonymous" => Some(CorsSettings::Anonymous),
        "use-credentials" => Some(CorsSettings::UseCredentials),
        _ => unreachable!(),
    })
}
