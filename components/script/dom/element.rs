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

use cssparser::match_ignore_ascii_case;
use devtools_traits::AttrInfo;
use dom_struct::dom_struct;
use embedder_traits::InputMethodType;
use euclid::default::{Rect, Size2D};
use html5ever::serialize::TraversalScope::{ChildrenOnly, IncludeNode};
use html5ever::{
    local_name, namespace_prefix, namespace_url, ns, LocalName, Namespace, Prefix, QualName,
};
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::HandleObject;
use net_traits::request::CorsSettings;
use net_traits::ReferrerPolicy;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::{BloomFilter, BLOOM_HASH_MASK};
use selectors::matching::{ElementSelectorFlags, MatchingContext};
use selectors::sink::Push;
use selectors::Element as SelectorsElement;
use servo_arc::Arc;
use servo_atoms::Atom;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::{AttrValue, LengthOrPercentageOrAuto};
use style::context::QuirksMode;
use style::invalidation::element::restyle_hints::RestyleHint;
use style::properties::longhands::{
    self, background_image, border_spacing, font_family, font_size,
};
use style::properties::{
    parse_style_attribute, ComputedValues, Importance, PropertyDeclaration,
    PropertyDeclarationBlock,
};
use style::rule_tree::CascadeLevel;
use style::selector_parser::{
    extended_filtering, NonTSPseudoClass, PseudoElement, RestyleDamage, SelectorImpl,
    SelectorParser,
};
use style::shared_lock::{Locked, SharedRwLock};
use style::stylesheets::layer_rule::LayerOrder;
use style::stylesheets::{CssRuleType, UrlExtraData};
use style::values::generics::position::PreferredRatio;
use style::values::generics::ratio::Ratio;
use style::values::generics::NonNegative;
use style::values::{computed, specified, AtomIdent, AtomString, CSSFloat};
use style::{dom_apis, thread_state, ArcSlice, CaseSensitivityExt};
use style_dom::ElementState;
use xml5ever::serialize::TraversalScope::{
    ChildrenOnly as XmlChildrenOnly, IncludeNode as XmlIncludeNode,
};

use super::customelementregistry::is_valid_custom_element_name;
use super::htmltablecolelement::{HTMLTableColElement, HTMLTableColElementLayoutHelpers};
use super::intersectionobserver::{IntersectionObserver, IntersectionObserverRegistration};
use crate::dom::activation::Activatable;
use crate::dom::attr::{Attr, AttrHelpersForLayout};
use crate::dom::bindings::cell::{ref_filter_map, DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::{ElementMethods, ShadowRootInit};
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::codegen::Bindings::HTMLTemplateElementBinding::HTMLTemplateElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMethods, ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::{
    ScrollBehavior, ScrollToOptions, WindowMethods,
};
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::conversions::DerivedFrom;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::xmlname::{
    matches_name_production, namespace_from_domstring, validate_and_extract,
};
use crate::dom::characterdata::CharacterData;
use crate::dom::create::create_element;
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementDefinition, CustomElementReaction, CustomElementState,
};
use crate::dom::document::{
    determine_policy_for_token, Document, LayoutDocumentHelpers, ReflowTriggerCondition,
};
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
use crate::dom::htmlhrelement::{HTMLHRElement, HTMLHRLayoutHelpers};
use crate::dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use crate::dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use crate::dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use crate::dom::htmllabelelement::HTMLLabelElement;
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlobjectelement::HTMLObjectElement;
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmloutputelement::HTMLOutputElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementLayoutHelpers};
use crate::dom::htmltableelement::{HTMLTableElement, HTMLTableElementLayoutHelpers};
use crate::dom::htmltablerowelement::{HTMLTableRowElement, HTMLTableRowElementLayoutHelpers};
use crate::dom::htmltablesectionelement::{
    HTMLTableSectionElement, HTMLTableSectionElementLayoutHelpers,
};
use crate::dom::htmltemplateelement::HTMLTemplateElement;
use crate::dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use crate::dom::htmlvideoelement::{HTMLVideoElement, LayoutHTMLVideoElementHelpers};
use crate::dom::mutationobserver::{Mutation, MutationObserver};
use crate::dom::namednodemap::NamedNodeMap;
use crate::dom::node::{
    BindContext, ChildrenMutation, LayoutNodeHelpers, Node, NodeDamage, NodeFlags, NodeTraits,
    ShadowIncluding, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::promise::Promise;
use crate::dom::raredata::ElementRareData;
use crate::dom::servoparser::ServoParser;
use crate::dom::shadowroot::{IsUserAgentWidget, ShadowRoot};
use crate::dom::text::Text;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::{vtable_for, VirtualMethods};
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;
use crate::stylesheet_loader::StylesheetOwner;
use crate::task::TaskOnce;

// TODO: Update focus state when the top-level browsing context gains or loses system focus,
// and when the element enters or leaves a browsing context container.
// https://html.spec.whatwg.org/multipage/#selector-focus

/// <https://dom.spec.whatwg.org/#element>
#[dom_struct]
pub(crate) struct Element {
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
    #[no_trace]
    is: DomRefCell<Option<LocalName>>,
    #[ignore_malloc_size_of = "Arc"]
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

impl fmt::Debug for DomRoot<Element> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
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

    impl_rare_data!(ElementRareData);

    pub(crate) fn restyle(&self, damage: NodeDamage) {
        let doc = self.node.owner_doc();
        let mut restyle = doc.ensure_pending_restyle(self);

        // FIXME(bholley): I think we should probably only do this for
        // NodeStyleDamaged, but I'm preserving existing behavior.
        restyle.hint.insert(RestyleHint::RESTYLE_SELF);

        if damage == NodeDamage::OtherNodeDamage {
            doc.note_node_with_dirty_descendants(self.upcast());
            restyle.damage = RestyleDamage::rebuild_and_reflow();
        }
    }

    pub(crate) fn set_is(&self, is: LocalName) {
        *self.is.borrow_mut() = Some(is);
    }

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

    // https://drafts.csswg.org/cssom-view/#potentially-scrollable
    fn is_potentially_scrollable_body(&self, can_gc: CanGc) -> bool {
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
                if !style.get_box().clone_overflow_x().is_scrollable() &&
                    !style.get_box().clone_overflow_y().is_scrollable()
                {
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
    pub(crate) fn attach_shadow(
        &self,
        // TODO: remove is_ua_widget argument
        is_ua_widget: IsUserAgentWidget,
        mode: ShadowRootMode,
        clonable: bool,
        slot_assignment_mode: SlotAssignmentMode,
    ) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1.
        // If element’s namespace is not the HTML namespace,
        // then throw a "NotSupportedError" DOMException.
        if self.namespace != ns!(html) {
            return Err(Error::NotSupported);
        }

        // Step 2.
        // If element’s local name is not a valid shadow host name,
        // then throw a "NotSupportedError" DOMException.
        if !is_valid_shadow_host_name(self.local_name()) {
            match self.local_name() {
                &local_name!("video") | &local_name!("audio")
                    if is_ua_widget == IsUserAgentWidget::Yes => {},
                _ => return Err(Error::NotSupported),
            }
        }

        // TODO: Update the following steps to align with the newer spec.
        // Step 3.
        if self.is_shadow_host() {
            return Err(Error::InvalidState);
        }

        // Steps 4, 5 and 6.
        let shadow_root = ShadowRoot::new(
            self,
            &self.node.owner_doc(),
            mode,
            slot_assignment_mode,
            clonable,
        );
        self.ensure_rare_data().shadow_root = Some(Dom::from_ref(&*shadow_root));
        shadow_root
            .upcast::<Node>()
            .set_containing_shadow_root(Some(&shadow_root));

        let bind_context = BindContext {
            tree_connected: self.upcast::<Node>().is_connected(),
            tree_is_in_a_document_tree: self.upcast::<Node>().is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: true,
        };
        shadow_root.bind_to_tree(&bind_context);

        let node = self.upcast::<Node>();
        node.dirty(NodeDamage::OtherNodeDamage);
        node.rev_version();

        Ok(shadow_root)
    }

    pub(crate) fn detach_shadow(&self) {
        let Some(ref shadow_root) = self.shadow_root() else {
            unreachable!("Trying to detach a non-attached shadow root");
        };

        let node = self.upcast::<Node>();
        node.note_dirty_descendants();
        node.rev_version();

        shadow_root.detach();
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

    /// Add a new IntersectionObserverRegistration to the element.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn add_intersection_observer_registration(
        &self,
        registration: IntersectionObserverRegistration,
    ) {
        self.ensure_rare_data()
            .registered_intersection_observers
            .push(registration);
    }

    /// Removes a certain IntersectionObserver.
    pub(crate) fn remove_intersection_observer(&self, observer: &IntersectionObserver) {
        self.ensure_rare_data()
            .registered_intersection_observers
            .retain(|reg_obs| *reg_obs.observer != *observer)
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
    fn has_class_for_layout(self, name: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool;
    fn get_classes_for_layout(self) -> Option<&'dom [Atom]>;

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
    fn has_class_for_layout(self, name: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        get_attr_for_layout(self, &ns!(), &local_name!("class")).is_some_and(|attr| {
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

    fn get_lang_for_layout(self) -> String {
        let mut current_node = Some(self.upcast::<Node>());
        while let Some(node) = current_node {
            current_node = node.composed_parent_node_ref();
            match node.downcast::<Element>() {
                Some(elem) => {
                    if let Some(attr) =
                        elem.get_attr_val_for_layout(&ns!(xml), &local_name!("lang"))
                    {
                        return attr.to_owned();
                    }
                    if let Some(attr) = elem.get_attr_val_for_layout(&ns!(), &local_name!("lang")) {
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
        self.push_attribute(&attr);
    }

    pub(crate) fn push_attribute(&self, attr: &Attr) {
        let name = attr.local_name().clone();
        let namespace = attr.namespace().clone();
        let mutation = LazyCell::new(|| Mutation::Attribute {
            name: name.clone(),
            namespace: namespace.clone(),
            old_value: None,
        });

        MutationObserver::queue_a_mutation_record(&self.node, mutation);

        if self.get_custom_element_definition().is_some() {
            let value = DOMString::from(&**attr.value());
            let reaction = CallbackReaction::AttributeChanged(name, None, Some(value), namespace);
            ScriptThread::enqueue_callback_reaction(self, reaction, None);
        }

        assert!(attr.GetOwnerElement().as_deref() == Some(self));
        self.will_mutate_attr(attr);
        self.attrs.borrow_mut().push(Dom::from_ref(attr));
        if attr.namespace() == &ns!() {
            vtable_for(self.upcast()).attribute_mutated(attr, AttributeMutation::Set(None));
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
                    Some(ref attr) => matches!(*attr.value(), AttrValue::Atom(_)),
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
            attr.set_value(value, self);
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
        if *namespace == ns!() {
            vtable_for(self.upcast()).parse_plain_attribute(local_name, value)
        } else {
            AttrValue::String(value.into())
        }
    }

    pub(crate) fn remove_attribute(
        &self,
        namespace: &Namespace,
        local_name: &LocalName,
    ) -> Option<DomRoot<Attr>> {
        self.remove_first_matching_attribute(|attr| {
            attr.namespace() == namespace && attr.local_name() == local_name
        })
    }

    pub(crate) fn remove_attribute_by_name(&self, name: &LocalName) -> Option<DomRoot<Attr>> {
        self.remove_first_matching_attribute(|attr| attr.name() == name)
    }

    fn remove_first_matching_attribute<F>(&self, find: F) -> Option<DomRoot<Attr>>
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

            let reaction =
                CallbackReaction::AttributeChanged(name, Some(old_value), None, namespace);
            ScriptThread::enqueue_callback_reaction(self, reaction, None);

            self.attrs.borrow_mut().remove(idx);
            attr.set_owner(None);
            if attr.namespace() == &ns!() {
                vtable_for(self.upcast()).attribute_mutated(&attr, AttributeMutation::Removed);
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
            self.remove_attribute(&ns!(), local_name);
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
                self.remove_attribute(&ns!(), local_name);
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
        assert!(local_name
            .chars()
            .all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch));
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
        assert!(local_name
            .chars()
            .all(|ch| !ch.is_ascii() || ch.to_ascii_lowercase() == ch));
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

    // https://dom.spec.whatwg.org/#insert-adjacent
    pub(crate) fn insert_adjacent(
        &self,
        where_: AdjacentPosition,
        node: &Node,
    ) -> Fallible<Option<DomRoot<Node>>> {
        let self_node = self.upcast::<Node>();
        match where_ {
            AdjacentPosition::BeforeBegin => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, Some(self_node)).map(Some)
                } else {
                    Ok(None)
                }
            },
            AdjacentPosition::AfterBegin => {
                Node::pre_insert(node, self_node, self_node.GetFirstChild().as_deref()).map(Some)
            },
            AdjacentPosition::BeforeEnd => Node::pre_insert(node, self_node, None).map(Some),
            AdjacentPosition::AfterEnd => {
                if let Some(parent) = self_node.GetParentNode() {
                    Node::pre_insert(node, &parent, self_node.GetNextSibling().as_deref()).map(Some)
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
        let new_children = ServoParser::parse_html_fragment(self, markup, can_gc);
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
            fragment.upcast::<Node>().AppendChild(&child).unwrap();
        }
        // Step 5.
        Ok(fragment)
    }

    pub(crate) fn fragment_parsing_context(
        owner_doc: &Document,
        element: Option<&Self>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        match element {
            Some(elem)
                if elem.local_name() != &local_name!("html") ||
                    !elem.html_element_in_html_document() =>
            {
                DomRoot::from_ref(elem)
            },
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

    pub(crate) fn ensure_element_internals(&self) -> DomRoot<ElementInternals> {
        let mut rare_data = self.ensure_rare_data();
        DomRoot::from_ref(rare_data.element_internals.get_or_insert_with(|| {
            let elem = self
                .downcast::<HTMLElement>()
                .expect("ensure_element_internals should only be called for an HTMLElement");
            Dom::from_ref(&*ElementInternals::new(elem))
        }))
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
    fn ClassList(&self) -> DomRoot<DOMTokenList> {
        self.class_list
            .or_init(|| DOMTokenList::new(self, &local_name!("class"), None))
    }

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_getter!(Slot, "slot");

    // https://dom.spec.whatwg.org/#dom-element-slot
    make_setter!(SetSlot, "slot");

    // https://dom.spec.whatwg.org/#dom-element-attributes
    fn Attributes(&self) -> DomRoot<NamedNodeMap> {
        self.attr_list
            .or_init(|| NamedNodeMap::new(&self.owner_window(), self))
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
                    self.remove_attribute_by_name(&name);
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
    fn SetAttributeNode(&self, attr: &Attr) -> Fallible<Option<DomRoot<Attr>>> {
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
            if self.get_custom_element_definition().is_some() {
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
            if attr.namespace() == &ns!() {
                vtable.attribute_mutated(attr, AttributeMutation::Set(Some(&old_attr.value())));
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
    fn SetAttributeNodeNS(&self, attr: &Attr) -> Fallible<Option<DomRoot<Attr>>> {
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
    fn RemoveAttributeNode(&self, attr: &Attr) -> Fallible<DomRoot<Attr>> {
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
    fn GetElementsByTagName(&self, localname: DOMString) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_qualified_name(&window, self.upcast(), LocalName::from(&*localname))
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbytagnamens
    fn GetElementsByTagNameNS(
        &self,
        maybe_ns: Option<DOMString>,
        localname: DOMString,
    ) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_tag_name_ns(&window, self.upcast(), localname, maybe_ns)
    }

    // https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    fn GetElementsByClassName(&self, classes: DOMString) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::by_class_name(&window, self.upcast(), classes)
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
    fn ScrollTo(&self, options: &ScrollToOptions) {
        self.Scroll(options, CanGc::note());
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollto
    fn ScrollTo_(&self, x: f64, y: f64) {
        self.Scroll_(x, y, CanGc::note());
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
        let point = node.scroll_offset();
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
        let point = node.scroll_offset();
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

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn GetInnerHTML(&self) -> Fallible<DOMString> {
        let qname = QualName::new(
            self.prefix().clone(),
            self.namespace().clone(),
            self.local_name().clone(),
        );

        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>()
                .html_serialize(ChildrenOnly(Some(qname)))
        } else {
            self.upcast::<Node>()
                .xml_serialize(XmlChildrenOnly(Some(qname)))
        };

        Ok(result)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-innerhtml>
    fn SetInnerHTML(&self, value: DOMString, can_gc: CanGc) -> ErrorResult {
        // Step 2.
        // https://github.com/w3c/DOM-Parsing/issues/1
        let target = if let Some(template) = self.downcast::<HTMLTemplateElement>() {
            DomRoot::upcast(template.Content(can_gc))
        } else {
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

        // Step 1.
        let frag = self.parse_fragment(value, can_gc)?;

        Node::replace_all(Some(frag.upcast()), &target);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn GetOuterHTML(&self) -> Fallible<DOMString> {
        let result = if self.owner_document().is_html_document() {
            self.upcast::<Node>().html_serialize(IncludeNode)
        } else {
            self.upcast::<Node>().xml_serialize(XmlIncludeNode)
        };

        Ok(result)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-element-outerhtml>
    fn SetOuterHTML(&self, value: DOMString, can_gc: CanGc) -> ErrorResult {
        let context_document = self.owner_document();
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

        // Step 5.
        let frag = parent.parse_fragment(value, can_gc)?;
        // Step 6.
        context_parent.ReplaceChild(frag.upcast(), context_node)?;
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
    fn Children(&self) -> DomRoot<HTMLCollection> {
        let window = self.owner_window();
        HTMLCollection::children(&window, self.upcast())
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
    fn Remove(&self) {
        self.upcast::<Node>().remove_self();
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

        Ok(dom_apis::element_matches(&element, &selectors, quirks_mode))
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
            DomRoot::from_ref(self),
            &selectors,
            quirks_mode,
        ))
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacentelement
    fn InsertAdjacentElement(
        &self,
        where_: DOMString,
        element: &Element,
    ) -> Fallible<Option<DomRoot<Element>>> {
        let where_ = where_.parse::<AdjacentPosition>()?;
        let inserted_node = self.insert_adjacent(where_, element.upcast())?;
        Ok(inserted_node.map(|node| DomRoot::downcast(node).unwrap()))
    }

    // https://dom.spec.whatwg.org/#dom-element-insertadjacenttext
    fn InsertAdjacentText(&self, where_: DOMString, data: DOMString, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let text = Text::new(data, &self.owner_document(), can_gc);

        // Step 2.
        let where_ = where_.parse::<AdjacentPosition>()?;
        self.insert_adjacent(where_, text.upcast()).map(|_| ())
    }

    // https://w3c.github.io/DOM-Parsing/#dom-element-insertadjacenthtml
    fn InsertAdjacentHTML(
        &self,
        position: DOMString,
        text: DOMString,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Step 1.
        let position = position.parse::<AdjacentPosition>()?;

        let context = match position {
            AdjacentPosition::BeforeBegin | AdjacentPosition::AfterEnd => {
                match self.upcast::<Node>().GetParentNode() {
                    Some(ref node) if node.is::<Document>() => {
                        return Err(Error::NoModificationAllowed)
                    },
                    None => return Err(Error::NoModificationAllowed),
                    Some(node) => node,
                }
            },
            AdjacentPosition::AfterBegin | AdjacentPosition::BeforeEnd => {
                DomRoot::from_ref(self.upcast::<Node>())
            },
        };

        // Step 2.
        let context = Element::fragment_parsing_context(
            &context.owner_doc(),
            context.downcast::<Element>(),
            can_gc,
        );

        // Step 3.
        let fragment = context.parse_fragment(text, can_gc)?;

        // Step 4.
        self.insert_adjacent(position, fragment.upcast())
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
    fn AttachShadow(&self, init: &ShadowRootInit) -> Fallible<DomRoot<ShadowRoot>> {
        // Step 1. Run attach a shadow root with this, init["mode"], init["clonable"], init["serializable"],
        // init["delegatesFocus"], and init["slotAssignment"].
        let shadow_root = self.attach_shadow(
            IsUserAgentWidget::No,
            init.mode,
            init.clonable,
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
}

impl VirtualMethods for Element {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<Node>() as &dyn VirtualMethods)
    }

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        // FIXME: This should be more fine-grained, not all elements care about these.
        if attr.local_name() == &local_name!("width") || attr.local_name() == &local_name!("height")
        {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        let node = self.upcast::<Node>();
        let doc = node.owner_doc();
        match attr.local_name() {
            &local_name!("tabindex") | &local_name!("draggable") | &local_name!("hidden") => {
                self.update_sequentially_focusable_status(CanGc::note())
            },
            &local_name!("style") => {
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
                            Arc::new(doc.style_shared_lock().wrap(parse_style_attribute(
                                &attr.value(),
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
            },
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
                                    shadow_root.unregister_element_id(self, old_value);
                                } else {
                                    doc.unregister_element_id(self, old_value);
                                }
                            }
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.register_element_id(self, value);
                                } else {
                                    doc.register_element_id(self, value);
                                }
                            }
                        },
                        AttributeMutation::Removed => {
                            if value != atom!("") {
                                if let Some(ref shadow_root) = containing_shadow_root {
                                    shadow_root.unregister_element_id(self, value);
                                } else {
                                    doc.unregister_element_id(self, value);
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
                    node.dirty(NodeDamage::OtherNodeDamage);
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
            local_name!("class") => AttrValue::from_serialized_tokenlist(value.into()),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if let Some(f) = self.as_maybe_form_control() {
            f.bind_form_control_to_tree();
        }

        let doc = self.owner_document();

        if let Some(ref shadow_root) = self.shadow_root() {
            shadow_root.bind_to_tree(context);
        }

        if !context.is_in_tree() {
            return;
        }

        self.update_sequentially_focusable_status(CanGc::note());

        if let Some(ref id) = *self.id_attribute.borrow() {
            if let Some(shadow_root) = self.containing_shadow_root() {
                shadow_root.register_element_id(self, id.clone());
            } else {
                doc.register_element_id(self, id.clone());
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

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if let Some(f) = self.as_maybe_form_control() {
            // TODO: The valid state of ancestors might be wrong if the form control element
            // has a fieldset ancestor, for instance: `<form><fieldset><input>`,
            // if `<input>` is unbound, `<form><fieldset>` should trigger a call to `update_validity()`.
            f.unbind_form_control_from_tree();
        }

        if !context.tree_is_in_a_document_tree && !context.tree_is_in_a_shadow_tree {
            return;
        }

        self.update_sequentially_focusable_status(CanGc::note());

        let doc = self.owner_document();

        let fullscreen = doc.GetFullscreenElement();
        if fullscreen.as_deref() == Some(self) {
            doc.exit_fullscreen(CanGc::note());
        }
        if let Some(ref value) = *self.id_attribute.borrow() {
            if let Some(ref shadow_root) = self.containing_shadow_root() {
                // Only unregister the element id if the node was disconnected from it's shadow root
                // (as opposed to the whole shadow tree being disconnected as a whole)
                if !self.upcast::<Node>().is_in_a_shadow_tree() {
                    shadow_root.unregister_element_id(self, value.clone());
                }
            } else {
                doc.unregister_element_id(self, value.clone());
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
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        } else {
            if flags.intersects(ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
                if let Some(next_child) = mutation.next_child() {
                    for child in next_child.inclusively_following_siblings() {
                        if child.is::<Element>() {
                            child.dirty(NodeDamage::OtherNodeDamage);
                        }
                    }
                }
            }
            if flags.intersects(ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR) {
                if let Some(child) = mutation.modified_edge_element() {
                    child.dirty(NodeDamage::OtherNodeDamage);
                }
            }
        }
    }

    fn adopting_steps(&self, old_doc: &Document) {
        self.super_type().unwrap().adopting_steps(old_doc);

        if self.owner_document().is_html_document() != old_doc.is_html_document() {
            self.tag_name.clear();
        }
    }
}

impl SelectorsElement for DomRoot<Element> {
    type Impl = SelectorImpl;

    #[allow(unsafe_code)]
    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*self.reflector().get_jsobject().get() })
    }

    fn parent_element(&self) -> Option<DomRoot<Element>> {
        self.upcast::<Node>().GetParentElement()
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

    fn prev_sibling_element(&self) -> Option<DomRoot<Element>> {
        self.node
            .preceding_siblings()
            .filter_map(DomRoot::downcast)
            .next()
    }

    fn next_sibling_element(&self) -> Option<DomRoot<Element>> {
        self.node
            .following_siblings()
            .filter_map(DomRoot::downcast)
            .next()
    }

    fn first_element_child(&self) -> Option<DomRoot<Element>> {
        self.GetFirstElementChild()
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
            NonTSPseudoClass::Lang(ref lang) => extended_filtering(&self.get_lang(), lang),

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

    fn is_part(&self, _name: &AtomIdent) -> bool {
        false
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
                Dom::from_ref(self.deref())
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
                    Dom::from_ref(p.deref())
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
            if matches!(
                doc.needs_reflow(),
                None | Some(ReflowTriggerCondition::PaintPostponed)
            ) {
                return rect;
            }
        }

        let mut rect = self.upcast::<Node>().client_rect(can_gc);
        let in_quirks_mode = doc.quirks_mode() == QuirksMode::Quirks;

        if (in_quirks_mode && doc.GetBody().as_deref() == self.downcast::<HTMLElement>()) ||
            (!in_quirks_mode && *self.root_element() == *self)
        {
            let viewport_dimensions = doc.window().window_size().initial_viewport.round().to_i32();
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

    pub(crate) fn is_invalid(&self, needs_update: bool) -> bool {
        if let Some(validatable) = self.as_maybe_validatable() {
            if needs_update {
                validatable
                    .validity_state()
                    .perform_validation_and_update(ValidationFlags::all());
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

    // https://html.spec.whatwg.org/multipage/#language
    pub(crate) fn get_lang(&self) -> String {
        self.upcast::<Node>()
            .inclusive_ancestors(ShadowIncluding::Yes)
            .filter_map(|node| {
                node.downcast::<Element>().and_then(|el| {
                    el.get_attribute(&ns!(xml), &local_name!("lang"))
                        .or_else(|| el.get_attribute(&ns!(), &local_name!("lang")))
                        .map(|attr| String::from(attr.Value()))
                })
                // TODO: Check meta tags for a pragma-set default language
                // TODO: Check HTTP Content-Language header
            })
            .next()
            .unwrap_or(String::new())
    }

    pub(crate) fn state(&self) -> ElementState {
        self.state.get()
    }

    pub(crate) fn set_state(&self, which: ElementState, value: bool) {
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
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
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
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
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
            promise.reject_error(Error::Type(String::from("fullscreen is not connected")));
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
        promise.resolve_native(&());
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
        self.promise.root().resolve_native(&());
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
            element.remove_attribute(&ns!(), &local_name!("crossorigin"));
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
