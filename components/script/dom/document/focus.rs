/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};
use std::cmp::Ordering;

use bitflags::bitflags;
use embedder_traits::FocusSequenceNumber;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::HTMLIFrameElementBinding::HTMLIFrameElementMethods;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;
use servo_base::id::BrowsingContextId;
use servo_constellation_traits::{
    RemoteFocusOperation, ScriptToConstellationMessage, SequentialFocusDirection,
};

use crate::dom::focusevent::FocusEventType;
use crate::dom::node::focus::FocusNavigationScopeOwner;
use crate::dom::types::{Element, EventTarget, FocusEvent, HTMLElement, HTMLIFrameElement, Window};
use crate::dom::{Document, Event, EventBubbles, EventCancelable, Node, NodeTraits};
use crate::realms::enter_realm;

/// The kind of focusable area a [`FocusableArea`] is. A [`FocusableArea`] may be click focusable,
/// sequentially focusable, or both.
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct FocusableAreaKind(u8);

bitflags! {
    impl FocusableAreaKind: u8 {
        /// <https://html.spec.whatwg.org/multipage/#click-focusable>
        ///
        /// > A focusable area is said to be click focusable if the user agent determines that it is
        /// > click focusable. User agents should consider focusable areas with non-null tabindex values
        /// > to be click focusable.
        const Click = 1 << 0;
        /// <https://html.spec.whatwg.org/multipage/#sequentially-focusable>.
        ///
        /// > A focusable area is said to be sequentially focusable if it is included in its
        /// > Document's sequential focus navigation order and the user agent determines that it is
        /// > sequentially focusable.
        const Sequential = 1 << 1;
    }
}

/// <https://html.spec.whatwg.org/multipage/#focusable-area>
#[derive(Clone, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum FocusableArea {
    Node {
        node: DomRoot<Node>,
        kind: FocusableAreaKind,
    },
    /// The viewport of an `<iframe>` element in its containing `Document`. `<iframe>`s
    /// are focusable areas, but have special behavior when focusing.
    IFrameViewport {
        iframe_element: DomRoot<HTMLIFrameElement>,
        kind: FocusableAreaKind,
    },
    #[default]
    Viewport,
}

impl std::fmt::Debug for FocusableArea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node { node, kind } => f
                .debug_struct("Node")
                .field("node", node)
                .field("kind", kind)
                .finish(),
            Self::IFrameViewport {
                iframe_element,
                kind,
            } => f
                .debug_struct("IFrameViewport")
                .field("pipeline", &iframe_element.pipeline_id())
                .field("kind", kind)
                .finish(),
            Self::Viewport => write!(f, "Viewport"),
        }
    }
}

impl FocusableArea {
    pub(crate) fn kind(&self) -> FocusableAreaKind {
        match self {
            Self::Node { kind, .. } | Self::IFrameViewport { kind, .. } => *kind,
            Self::Viewport => FocusableAreaKind::Click | FocusableAreaKind::Sequential,
        }
    }

    /// If this focusable area is a node, return it as an [`Element`] if it is possible, otherwise
    /// return `None`. This is the [`Element`] to use for applying `:focus` state and for firing
    /// `blur` and `focus` events if any.
    ///
    /// Note: This is currently in a transitional state while the code moves more toward the
    /// specification.
    pub(crate) fn element(&self) -> Option<&Element> {
        match self {
            Self::Node { node, .. } => node.downcast(),
            Self::IFrameViewport { iframe_element, .. } => Some(iframe_element.upcast()),
            Self::Viewport => None,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-anchor>
    pub(crate) fn dom_anchor(&self, document: &Document) -> DomRoot<Node> {
        match self {
            Self::Node { node, .. } => node.clone(),
            Self::IFrameViewport { iframe_element, .. } => {
                DomRoot::from_ref(iframe_element.upcast())
            },
            Self::Viewport => DomRoot::from_ref(document.upcast()),
        }
    }

    pub(crate) fn focus_chain(&self) -> Vec<FocusableArea> {
        match self {
            FocusableArea::Node { .. } | FocusableArea::IFrameViewport { .. } => {
                vec![self.clone(), FocusableArea::Viewport]
            },
            FocusableArea::Viewport => vec![self.clone()],
        }
    }
}

/// The [`DocumentFocusHandler`] is a structure responsible for handling and storing data related to
/// focus for the `Document`. It exists to decrease the size of the `Document`.
/// structure.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentFocusHandler {
    /// The [`Window`] element for this [`DocumentFocusHandler`].
    window: Dom<Window>,
    /// The focused area of the [`Document`].
    ///
    /// <https://html.spec.whatwg.org/multipage/#focused-area-of-the-document>
    focused_area: DomRefCell<FocusableArea>,
    /// The last sequence number sent to the constellation.
    #[no_trace]
    focus_sequence: Cell<FocusSequenceNumber>,
    /// Indicates whether the container is included in the top-level browsing
    /// context's focus chain (not considering system focus). Permanently `true`
    /// for a top-level document.
    has_focus: Cell<bool>,
}

impl DocumentFocusHandler {
    pub(crate) fn new(window: &Window, has_focus: bool) -> Self {
        Self {
            window: Dom::from_ref(window),
            focused_area: Default::default(),
            focus_sequence: Cell::new(FocusSequenceNumber::default()),
            has_focus: Cell::new(has_focus),
        }
    }

    pub(crate) fn has_focus(&self) -> bool {
        self.has_focus.get()
    }

    pub(crate) fn set_has_focus(&self, has_focus: bool) {
        self.has_focus.set(has_focus);
    }

    /// Return the element that currently has focus. If `None` is returned the viewport itself has focus.
    pub(crate) fn focused_area<'a>(&'a self) -> Ref<'a, FocusableArea> {
        let focused_area = self.focused_area.borrow();
        Ref::map(focused_area, |focused_area| focused_area)
    }

    /// Set the element that currently has focus and update the focus state for both the previously
    /// set element (if any) and the new one, as well as the new one. This will not do anything if
    /// the new element is the same as the previous one. Note that this *will not* fire any focus
    /// events. If that is necessary the [`DocumentFocusHandler::focus`] should be used.
    pub(crate) fn set_focused_area(&self, new_focusable_area: FocusableArea) {
        if new_focusable_area == *self.focused_area.borrow() {
            return;
        }

        // From <https://html.spec.whatwg.org/multipage/#selector-focus>
        // > For the purposes of the CSS :focus pseudo-class, an element has the focus when:
        // >  - it is not itself a navigable container; and
        // >  - any of the following are true:
        // >    - it is one of the elements listed in the current focus chain of the top-level
        // >      traversable; or
        // >    - its shadow root shadowRoot is not null and shadowRoot is the root of at least one
        // >      element that has the focus.
        //
        // We are trying to accomplish the last requirement here, by walking up the tree and
        // marking each shadow host as focused.
        fn recursively_set_focus_status(element: &Element, new_state: bool) {
            element.set_focus_state(new_state);

            let Some(shadow_root) = element.containing_shadow_root() else {
                return;
            };
            recursively_set_focus_status(&shadow_root.Host(), new_state);
        }

        if let Some(previously_focused_element) = self.focused_area.borrow().element() {
            recursively_set_focus_status(previously_focused_element, false);
        }
        if let Some(newly_focused_element) = new_focusable_area.element() {
            recursively_set_focus_status(newly_focused_element, true);
        }

        *self.focused_area.borrow_mut() = new_focusable_area;
    }

    /// Get the last sequence number sent to the constellation.
    ///
    /// Received focus-related messages with sequence numbers less than the one
    /// returned by this method must be discarded.
    pub fn focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.get()
    }

    /// Generate the next sequence number for focus-related messages.
    fn increment_fetch_focus_sequence(&self) -> FocusSequenceNumber {
        self.focus_sequence.set(FocusSequenceNumber(
            self.focus_sequence
                .get()
                .0
                .checked_add(1)
                .expect("too many focus messages have been sent"),
        ));
        self.focus_sequence.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#current-focus-chain-of-a-top-level-traversable>
    pub(crate) fn current_focus_chain(&self) -> Vec<FocusableArea> {
        // > The current focus chain of a top-level traversable is the focus chain of the
        // > currently focused area of traversable, if traversable is non-null, or an empty list
        // > otherwise.

        // We cannot easily get the full focus chain of the top-level traversable, so we just
        // get the bits that intersect with this `Document`. The rest will be handled
        // internally in [`Self::focus_update_steps`].
        if !self.has_focus() {
            return vec![];
        }
        self.focused_area().focus_chain()
    }

    /// Reassign the focus context to the element that last requested focus during this
    /// transaction, or the document if no elements requested it.
    pub(crate) fn focus(&self, cx: &mut JSContext, new_focus_target: FocusableArea) {
        let old_focus_chain = self.current_focus_chain();
        let new_focus_chain = new_focus_target.focus_chain();
        self.focus_update_steps(cx, new_focus_chain, old_focus_chain, &new_focus_target);

        // Advertise the change in the focus chain.
        // <https://html.spec.whatwg.org/multipage/#focus-chain>
        // <https://html.spec.whatwg.org/multipage/#focusing-steps>
        //
        // TODO: Integrate this into the "focus update steps."
        //
        // If the top-level BC doesn't have system focus, this won't
        // have an immediate effect, but it will when we gain system
        // focus again. Therefore we still have to send `ScriptMsg::
        // Focus`.
        //
        // When a container with a non-null nested browsing context is
        // focused, its active document becomes the focused area of the
        // top-level browsing context instead. Therefore we need to let
        // the constellation know if such a container is focused.
        //
        // > The focusing steps for an object `new focus target` [...]
        // >
        // >  3. If `new focus target` is a browsing context container
        // >     with non-null nested browsing context, then set
        // >     `new focus target` to the nested browsing context's
        // >     active document.
        let child_browsing_context_id = match new_focus_target {
            FocusableArea::IFrameViewport { iframe_element, .. } => {
                iframe_element.browsing_context_id()
            },
            _ => None,
        };
        let sequence = self.increment_fetch_focus_sequence();

        debug!(
            "Advertising the focus request to the constellation \
                        with sequence number {sequence:?} and child \
                        {child_browsing_context_id:?}",
        );
        self.window.send_to_constellation(
            ScriptToConstellationMessage::FocusAncestorBrowsingContextsForFocusingSteps(
                child_browsing_context_id,
                sequence,
            ),
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-update-steps>
    pub(crate) fn focus_update_steps(
        &self,
        cx: &mut JSContext,
        mut new_focus_chain: Vec<FocusableArea>,
        mut old_focus_chain: Vec<FocusableArea>,
        new_focus_target: &FocusableArea,
    ) {
        let new_focus_chain_was_empty = new_focus_chain.is_empty();

        // Step 1: If the last entry in old chain and the last entry in new chain are the same,
        // pop the last entry from old chain and the last entry from new chain and redo this
        // step.
        //
        // We avoid recursion here.
        while let (Some(last_new), Some(last_old)) =
            (new_focus_chain.last(), old_focus_chain.last())
        {
            if last_new == last_old {
                new_focus_chain.pop();
                old_focus_chain.pop();
            } else {
                break;
            }
        }

        // If the two focus chains are both empty, focus hasn't changed. This isn't in the
        // specification, but we must do it because we set the focused area to the viewport
        // before blurring. If no focus changes, that would mean the currently focused element
        // loses focus.
        if old_focus_chain.is_empty() && new_focus_chain.is_empty() {
            return;
        }
        // Although the "focusing steps" in the HTML specification say to wait until after firing
        // the "blur" event to change the currently focused area of the Document, browsers tend
        // to set it to the viewport before firing the "blur" event.
        //
        // See https://github.com/whatwg/html/issues/1569
        self.set_focused_area(FocusableArea::Viewport);

        // Step 2: For each entry entry in old chain, in order, run these substeps:
        // Note: `old_focus_chain` might be empty!
        let last_old_focus_chain_entry = old_focus_chain.len().saturating_sub(1);
        for (index, entry) in old_focus_chain.iter().enumerate() {
            // Step 2.1: If entry is an input element, and the change event applies to the element,
            // and the element does not have a defined activation behavior, and the user has
            // changed the element's value or its list of selected files while the control was
            // focused without committing that change (such that it is different to what it was
            // when the control was first focused), then:
            // Step 2.1.1: Set entry's user validity to true.
            // Step 2.1.2: Fire an event named change at the element, with the bubbles attribute initialized to true.
            // TODO: Implement this.

            // Step 2.2:
            // - If entry is an element, let blur event target be entry.
            // - If entry is a Document object, let blur event target be that Document object's
            //    relevant global object.
            // - Otherwise, let blur event target be null.
            //
            // Note: We always send focus and blur events for `<iframe>` elements, but other
            // browsers only seem to do that conditionally. This needs a bit more research.
            let blur_event_target = match entry {
                FocusableArea::Node { node, .. } => Some(node.upcast::<EventTarget>()),
                FocusableArea::IFrameViewport { iframe_element, .. } => {
                    Some(iframe_element.upcast())
                },
                FocusableArea::Viewport => Some(self.window.upcast::<EventTarget>()),
            };

            // Step 2.3: If entry is the last entry in old chain, and entry is an Element, and
            // the last entry in new chain is also an Element, then let related blur target be
            // the last entry in new chain. Otherwise, let related blur target be null.
            //
            // Note: This can only happen when the focused `Document` doesn't change and we are
            // moving focus from one element to another. These elements are the last in the chain
            // because of the popping we do at the start of these steps.
            let related_blur_target = match new_focus_chain.last() {
                Some(FocusableArea::Node { node, .. })
                    if index == last_old_focus_chain_entry &&
                        matches!(entry, FocusableArea::Node { .. }) =>
                {
                    Some(node.upcast())
                },
                _ => None,
            };

            // Step 2.4: If blur event target is not null, fire a focus event named blur at
            // blur event target, with related blur target as the related target.
            if let Some(blur_event_target) = blur_event_target {
                self.fire_focus_event(
                    cx,
                    FocusEventType::Blur,
                    blur_event_target,
                    related_blur_target,
                );
            }
        }

        // Step 3: Apply any relevant platform-specific conventions for focusing new focus
        // target. (For example, some platforms select the contents of a text control when that
        // control is focused.)
        if &*self.focused_area() != new_focus_target &&
            let Some(html_element) = new_focus_target
                .element()
                .and_then(|element| element.downcast::<HTMLElement>())
        {
            html_element.handle_focus_state_for_contenteditable(cx);
        }

        self.set_has_focus(!new_focus_chain_was_empty);

        // Step 4: For each entry entry in new chain, in reverse order, run these substeps:
        // Note: `new_focus_chain` might be empty!
        let last_new_focus_chain_entry = new_focus_chain.len().saturating_sub(1); // Might be empty, so calculated here.
        for (index, entry) in new_focus_chain.iter().enumerate().rev() {
            // Step 4.1: If entry is a focusable area, and the focused area of the document is
            // not entry:
            //
            // Here we deviate from the specification a bit, as all focus chain elements are
            // focusable areas currently. We just assume that it means the first entry of the
            // chain, which is the new focus target
            if index == 0 {
                // Step 4.1.1: Set document's relevant global object's navigation API's focus
                // changed during ongoing navigation to true.
                // TODO: Implement this.

                // Step 4.1.2: Designate entry as the focused area of the document.
                self.set_focused_area(entry.clone());
            }

            // Step 4.2:
            // - If entry is an element, let focus event target be entry.
            // - If entry is a Document object, let focus event target be that Document
            //   object's relevant global object.
            // - Otherwise, let focus event target be null.
            //
            // Note: We always send focus and blur events for `<iframe>` elements, but other
            // browsers only seem to do that conditionally. This needs a bit more research.
            let focus_event_target = match entry {
                FocusableArea::Node { node, .. } => Some(node.upcast::<EventTarget>()),
                FocusableArea::IFrameViewport { iframe_element, .. } => {
                    Some(iframe_element.upcast())
                },
                FocusableArea::Viewport => Some(self.window.upcast::<EventTarget>()),
            };

            // Step 4.3: If entry is the last entry in new chain, and entry is an Element, and
            // the last entry in old chain is also an Element, then let related focus target be
            // the last entry in old chain. Otherwise, let related focus target be null.
            //
            // Note: This can only happen when the focused `Document` doesn't change and we are
            // moving focus from one element to another. These elements are the last in the chain
            // because of the popping we do at the start of these steps.
            let related_focus_target = match old_focus_chain.last() {
                Some(FocusableArea::Node { node, .. })
                    if index == last_new_focus_chain_entry &&
                        matches!(entry, FocusableArea::Node { .. }) =>
                {
                    Some(node.upcast())
                },
                _ => None,
            };

            // Step 4.4: If focus event target is not null, fire a focus event named focus at
            // focus event target, with related focus target as the related target.
            if let Some(focus_event_target) = focus_event_target {
                self.fire_focus_event(
                    cx,
                    FocusEventType::Focus,
                    focus_event_target,
                    related_focus_target,
                );
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-focus-event>
    pub(crate) fn fire_focus_event(
        &self,
        cx: &mut JSContext,
        focus_event_type: FocusEventType,
        event_target: &EventTarget,
        related_target: Option<&EventTarget>,
    ) {
        let event_name = match focus_event_type {
            FocusEventType::Focus => "focus".into(),
            FocusEventType::Blur => "blur".into(),
        };

        let event = FocusEvent::new(
            &self.window,
            event_name,
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            Some(&self.window),
            0i32,
            related_target,
            CanGc::from_cx(cx),
        );
        let event = event.upcast::<Event>();
        event.set_trusted(true);
        event.fire(event_target, CanGc::from_cx(cx));
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-fixup-rule>
    /// > For each doc of docs, if the focused area of doc is not a focusable area, then run the
    /// > focusing steps for doc's viewport, and set doc's relevant global object's navigation API's
    /// > focus changed during ongoing navigation to false.
    ///
    /// TODO: Handle the "focus changed during ongoing navigation" flag.
    pub(crate) fn perform_focus_fixup_rule(&self, cx: &mut JSContext) {
        if self
            .focused_area
            .borrow()
            .element()
            .is_none_or(|focused| focused.is_focusable_area())
        {
            return;
        }
        self.focus(cx, FocusableArea::Viewport);
    }

    pub(crate) fn sequentially_focus_child_iframe_local_or_remote(
        &self,
        cx: &mut JSContext,
        iframe_element: &HTMLIFrameElement,
        direction: SequentialFocusDirection,
    ) {
        if let Some(content_document) = iframe_element.GetContentDocument() {
            // The <iframe> is in the same `ScriptThread` and we have direct access to it. We can
            // move the focus directly.
            content_document
                .focus_handler()
                .sequential_focus_from_another_document(cx, None, direction);
        } else if let Some(browsing_context_id) = iframe_element.browsing_context_id() {
            self.window.send_to_constellation(
                ScriptToConstellationMessage::FocusRemoteBrowsingContext(
                    browsing_context_id,
                    RemoteFocusOperation::Sequential(direction, None),
                ),
            );
        } else {
            iframe_element
                .upcast::<Node>()
                .run_the_focusing_steps(cx, None);
        }
    }

    pub(crate) fn sequentially_focus_parent_local_or_remote(
        &self,
        cx: &mut JSContext,
        direction: SequentialFocusDirection,
    ) {
        let window_proxy = self.window.window_proxy();
        if let Some(iframe) = window_proxy.frame_element() {
            // The parent browsing context is in the same `ScriptThread` and we have direct access
            // to it. We can move the focus directly.
            let browsing_context_id = iframe
                .downcast::<HTMLIFrameElement>()
                .and_then(|iframe_element| iframe_element.browsing_context_id());
            iframe
                .owner_document()
                .focus_handler()
                .sequential_focus_from_another_document(cx, browsing_context_id, direction);
        } else if let Some(browsing_context_id) = window_proxy
            .parent()
            .map(|parent| parent.browsing_context_id())
        {
            self.window.send_to_constellation(
                ScriptToConstellationMessage::FocusRemoteBrowsingContext(
                    browsing_context_id,
                    RemoteFocusOperation::Sequential(
                        direction,
                        Some(window_proxy.browsing_context_id()),
                    ),
                ),
            );
        }
    }

    pub(crate) fn sequential_focus_from_another_document(
        &self,
        cx: &mut JSContext,
        browsing_context_id: Option<BrowsingContextId>,
        direction: SequentialFocusDirection,
    ) {
        let _realm = enter_realm(&*self.window);
        let starting_point = browsing_context_id.and_then(|browsing_context_id| {
            self.window
                .Document()
                .iframes()
                .get(browsing_context_id)
                .map(|iframe| DomRoot::from_ref(iframe.element.upcast::<Node>()))
        });
        self.window
            .Document()
            .event_handler()
            .sequential_focus_navigation_loop(
                cx,
                starting_point,
                direction,
                true, /* allow focusing viewport */
            );
    }
}

/// <https://html.spec.whatwg.org/multipage/#selection-mechanism>
///
/// This also incorporates the case where the starting point is a navigable
/// as that is a distinct set of behaviors from the two kinds of mechanisms
/// listed in the specification.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SequentialFocusNavigationMechanism {
    Dom,
    Sequential(i32 /* focused_element_tab_index */),
    /// This case isn't mentioned explicitly in the specification, but it's implied. It works
    /// like `Sequential`, but without a starting point. This kind of search will return the
    /// first or last (depending on direction) sequentially focusable element in sequential
    /// focus order. This is used in two situations:
    ///
    ///  - When the starting point is a navigable
    ///  - When descending into a nested focus scope
    FirstOrLast,
}

#[derive(PartialEq)]
enum Continue {
    Yes,
    No,
}

#[derive(PartialEq)]
pub(crate) enum SequentialFocusNavigationSearchContext {
    /// The focus scope that initiated this search. Containing contexts and nested contexts can
    /// both be searched.
    Original,
    /// The search has descended into a nested search scope. Containing contexts should never
    /// be searched.
    Nested,
    /// The search has ascended to search a containing search scope. The starting point's focus
    /// scope should never be searched to avoid cycles.
    Containing,
}

/// This structure is used to do a traversal search of the DOM in order to find an
/// appropriate target when doing sequential focus navigation, such as when handling
/// tab key presses.
///
/// The specification talks about the [flattened tabindex-ordered focus navigation scope],
/// which represents all of the [tabindex-ordered focus navigation scope]s of a particular
/// page, flattened into a single list of all the sequentially focusable areas of the
/// page. Then, the specification describes how to search this list during sequential focus
/// navigation.
///
/// The choice that Servo and other browsers make is to trade updating this flattened list
/// during every DOM mutation (frequent) with a DOM traversal of, potentially, the entire
/// document during sequential focus navigation (infrequent).
///
/// The search done via [`SequentialFocusNavigationSearch`] matches the semantics of the
/// flattened tabindex-ordered focus navigation scope without having to maintain the
/// flattened list. It uses a series of nested traversals (one per focus scope) that
/// only considers each focusable area of a page at most once.
///
/// The search performs a linear DOM traversal starting at the containing focus scope of
/// the search start point. When encountering a nested focus scope, if that scope could
/// contain the final target for the search, the search recurses into the nested scope. If
/// the search reaches the end of a focus scope without finding a candidate, the search
/// continues in the focus scope's containing scope (though never re-ascending back into a
/// scope it recursed from).
///
/// [flattened tabindex-ordered focus navigation scope]: https://html.spec.whatwg.org/multipage/#flattened-tabindex-ordered-focus-navigation-scope
/// [tabindex-ordered focus navigation scope]: https://html.spec.whatwg.org/multipage/#tabindex-ordered-focus-navigation-scope
pub(crate) struct SequentialFocusNavigationSearch {
    focus_navigation_scope_owner: FocusNavigationScopeOwner,
    direction: SequentialFocusDirection,
    mechanism: SequentialFocusNavigationMechanism,
    starting_point: Option<DomRoot<Node>>,
    current_winner: Option<(DomRoot<Element>, i32)>,
    passed_starting_point: bool,
    search_context: SequentialFocusNavigationSearchContext,
}

impl SequentialFocusNavigationSearch {
    pub(crate) fn new(
        focus_navigation_scope_owner: FocusNavigationScopeOwner,
        direction: SequentialFocusDirection,
        mechanism: SequentialFocusNavigationMechanism,
        starting_point: Option<DomRoot<Node>>,
    ) -> Self {
        // If there's no starting point, the starting point is actually the root element, which
        // we always have passed.
        let passed_starting_point = starting_point.is_none();
        Self {
            focus_navigation_scope_owner,
            direction,
            mechanism,
            starting_point,
            current_winner: Default::default(),
            passed_starting_point,
            search_context: SequentialFocusNavigationSearchContext::Original,
        }
    }

    pub(crate) fn search(mut self) -> Option<DomRoot<Element>> {
        for node in self.focus_navigation_scope_owner.iterator() {
            if self.process_node(&node) == Continue::No {
                break;
            }
        }

        if let Some(winner) = self.current_winner.take() {
            return Some(winner.0);
        }

        // If searching a nested focus navigation scope, never try to search the containing
        // scope, as that will lead to an endless cycle.
        if self.search_context != SequentialFocusNavigationSearchContext::Nested {
            return self.maybe_search_in_containing_focus_navigation_scope();
        }

        None
    }

    fn maybe_search_in_containing_focus_navigation_scope(&self) -> Option<DomRoot<Element>> {
        let containing_node = self.focus_navigation_scope_owner.node();
        let containing_focus_navigation_scope_owner =
            containing_node.containing_focus_navigation_scope_owner()?;

        let tab_index = containing_node
            .downcast::<Element>()?
            .explicitly_set_tab_index()
            .unwrap_or_default();
        let mechanism = match &self.mechanism {
            // If the traversal was sequential, but the containing focus navigation scope owner was
            // explicitly marked as not sequentially focusable, the search in the containing scope
            // needs to work like a DOM traversal i.e. take the first sequentially focusable target
            // after this one in the parent traversal.
            SequentialFocusNavigationMechanism::Sequential(..) if tab_index == -1 => {
                SequentialFocusNavigationMechanism::Dom
            },
            SequentialFocusNavigationMechanism::Sequential(..) => {
                SequentialFocusNavigationMechanism::Sequential(tab_index)
            },
            mechanism => *mechanism,
        };

        if self.direction == SequentialFocusDirection::Backward &&
            let Some(containing_element) = containing_node.downcast::<Element>() &&
            containing_element.is_sequentially_focusable()
        {
            return Some(DomRoot::from_ref(containing_element));
        }

        Self {
            focus_navigation_scope_owner: containing_focus_navigation_scope_owner,
            direction: self.direction,
            mechanism,
            starting_point: Some(DomRoot::from_ref(containing_node)),
            current_winner: Default::default(),
            passed_starting_point: false,
            search_context: SequentialFocusNavigationSearchContext::Containing,
        }
        .search()
    }

    fn process_node(&mut self, node: &Node) -> Continue {
        if Some(node) == self.starting_point.as_deref() {
            self.passed_starting_point = true;
        } else if self.process_node_as_sequentially_focusable_node(node) == Continue::No {
            return Continue::No;
        }

        self.process_node_as_focus_scope_owner(node)
    }

    /// If this node is sequentially focusable, consider whether or not to accept it
    /// as the new winner.
    fn process_node_as_sequentially_focusable_node(&mut self, node: &Node) -> Continue {
        let Some(element) = node.downcast::<Element>() else {
            return Continue::Yes;
        };
        if !element.is_sequentially_focusable() {
            return Continue::Yes;
        }

        let tab_index = element.explicitly_set_tab_index().unwrap_or_default();
        let (is_new_winner, should_continue) = self.process_candidate_with_tab_index(tab_index);
        if is_new_winner {
            self.current_winner = Some((DomRoot::from_ref(element), tab_index));
        }
        should_continue
    }

    /// If this node itself forms a nested sequential focus scope, decide whether or
    /// not to descend and consider its contained focusable areas as candidates.
    fn process_node_as_focus_scope_owner(&mut self, node: &Node) -> Continue {
        // Never try to recurse into the same focus scope that we are in. This path
        // might be reached if we are in the root focus scope where the document is
        // one of the nodes processed.
        if self.focus_navigation_scope_owner.node() == node {
            return Continue::Yes;
        }

        // If the search has ascended into a containing scope, never try to search back down
        // into the scope that originated this part of the search. Otherwise the search would
        // cycle endlessly.
        if Some(node) == self.starting_point.as_deref() &&
            self.search_context == SequentialFocusNavigationSearchContext::Containing
        {
            return Continue::Yes;
        }

        let Some(focus_navigation_scope_owner) = node.as_focus_navigation_scope_owner() else {
            return Continue::Yes;
        };

        // The candidate inherits the tab index of the node that establishes its containing
        // sequential focus navigation scope.
        let tab_index = focus_navigation_scope_owner
            .node()
            .downcast::<Element>()
            .and_then(Element::explicitly_set_tab_index)
            .unwrap_or_default();
        let (is_new_winner, should_continue) = self.process_candidate_with_tab_index(tab_index);
        if !is_new_winner {
            return should_continue;
        }

        let mechanism = match self.mechanism {
            // If we were searching without regard to sequential focus order, keep doing that.
            SequentialFocusNavigationMechanism::Dom => SequentialFocusNavigationMechanism::Dom,
            // If we were searching taking into account sequential focus order, keep doing that, but
            // take the first candidate in sequential focus order without regard to the outer scope's
            // starting point or tab index.
            _ => SequentialFocusNavigationMechanism::FirstOrLast,
        };

        let element = Self {
            focus_navigation_scope_owner,
            direction: self.direction,
            mechanism,
            starting_point: None,
            current_winner: Default::default(),
            passed_starting_point: self.passed_starting_point,
            search_context: SequentialFocusNavigationSearchContext::Nested,
        }
        .search();

        let Some(element) = element else {
            return Continue::Yes;
        };

        self.current_winner = Some((element, tab_index));
        should_continue
    }

    /// Process the node or focus scope owner with the provided tab index according to this
    /// search's search mechanism. Returns a boolean that is true if this candidate is the new
    /// winner and a [`Continue`] which says whether to keep searching or stop.
    fn process_candidate_with_tab_index(&mut self, candidate_tab_index: i32) -> (bool, Continue) {
        match self.mechanism {
            SequentialFocusNavigationMechanism::Dom => self.process_element_for_dom_traversal(),
            SequentialFocusNavigationMechanism::Sequential(focused_element_tab_index) => self
                .process_element_for_sequential_traversal(
                    candidate_tab_index,
                    focused_element_tab_index,
                ),
            SequentialFocusNavigationMechanism::FirstOrLast => (
                self.process_element_for_first_or_last_traversal(candidate_tab_index),
                Continue::Yes,
            ),
        }
    }

    /// Process the node or focus scope owner, given the state of [`Self::passed_starting_point`]
    /// with for searches with the [`SequentialFocusNavigationMechanism::Dom`] search mechanism.
    /// Returns a boolean that is true if this candidate is the new winner and a [`Continue`] which
    /// says whether to keep searching or stop.
    fn process_element_for_dom_traversal(&self) -> (bool, Continue) {
        match self.direction {
            // direction is "forward"
            // > Let candidate be the first suitable sequentially focusable area after starting point,
            // > in starting point's Document's sequential focus navigation order, if any; or else
            // > null
            SequentialFocusDirection::Forward if self.passed_starting_point => (true, Continue::No),
            // If searching forward, do not consider anything until passing the starting point.
            SequentialFocusDirection::Forward => (false, Continue::Yes),
            // direction is "backward"
            // > Let candidate be the last suitable sequentially focusable area before starting
            // > point, in starting point's Document's sequential focus navigation order, if any; or
            // > else null
            SequentialFocusDirection::Backward if !self.passed_starting_point => {
                (true, Continue::Yes)
            },
            // There is no possible winner after the starting point when searching backward.
            SequentialFocusDirection::Backward => (false, Continue::No),
        }
    }

    /// Process the node or focus scope owner with the provided tab index for searches with the
    /// [`SequentialFocusNavigationMechanism::FirstOrLast`] search mechanism. Returns a boolean
    /// that is true if this candidate is the new winner.
    fn process_element_for_first_or_last_traversal(
        &self,
        candidate_element_tab_index: i32,
    ) -> bool {
        let Some((_, winning_tab_index)) = self.current_winner else {
            return true;
        };

        let candidate_and_current_winner_ordering =
            compare_tab_indices(candidate_element_tab_index, winning_tab_index);
        match self.direction {
            // direction is "forward"
            // > Let candidate be the first suitable sequentially focusable area in starting point's
            // > active document, if any; or else null
            //
            // There's an ambiguity in the specification here. In this case it says to choose
            // the "first suitable sequentially focusable area." It's possible to interpret this
            // as the first in DOM order, but browsers seem to agree to follow tab index
            // order instead.
            //
            // Pick the lowest, prioritizing the earlier node when equal.
            SequentialFocusDirection::Forward
                if candidate_and_current_winner_ordering == Ordering::Less =>
            {
                true
            },
            // direction is "backward"
            // > Let candidate be the last suitable sequentially focusable area in starting point's
            // > active document, if any; or else null
            //
            // There's an ambiguity in the specification here. In this case it says to choose
            // the "last suitable sequentially focusable area" It's possible to interpret this
            // as the first in DOM order, but browsers seem to agree to following tab index
            // order instead.
            //
            // Pick the highest, prioritizing the later node when equal.
            SequentialFocusDirection::Backward
                if candidate_and_current_winner_ordering != Ordering::Less =>
            {
                true
            },
            _ => false,
        }
    }

    /// Process the node or focus scope owner with the provided tab index for searches with the
    /// [`SequentialFocusNavigationMechanism::Sequential`] search mechanism. Returns a boolean
    /// that is true if this candidate is the new winner and a [`Continue`] which says whether to
    /// keep searching or stop.
    fn process_element_for_sequential_traversal(
        &self,
        candidate_element_tab_index: i32,
        focused_element_tab_index: i32,
    ) -> (bool, Continue) {
        let candidate_and_focused_ordering =
            compare_tab_indices(candidate_element_tab_index, focused_element_tab_index);
        match self.direction {
            SequentialFocusDirection::Forward => {
                // If moving forward the first element with equal tab index after the current
                // element is the winner.
                if self.passed_starting_point && candidate_and_focused_ordering == Ordering::Equal {
                    return (true, Continue::No);
                }
                // If the candidate element does not have a greater tab index, then discard it.
                if candidate_and_focused_ordering != Ordering::Greater {
                    return (false, Continue::Yes);
                }
                let Some((_, winning_tab_index)) = self.current_winner else {
                    // If this candidate has a tab index which is one greater than the current
                    // tab index, then we know it is the winner, because we give precedence to
                    // elements earlier in the DOM.
                    if candidate_element_tab_index == focused_element_tab_index + 1 {
                        return (true, Continue::No);
                    }

                    return (true, Continue::Yes);
                };

                // If the candidate element has a lesser tab index than the current winner,
                // then it becomes the winner.
                let should_select =
                    compare_tab_indices(candidate_element_tab_index, winning_tab_index) ==
                        Ordering::Less;

                (should_select, Continue::Yes)
            },
            SequentialFocusDirection::Backward => {
                // If moving backward the last element with an equal tab index that precedes
                // the focused element in the DOM is the winner.
                if !self.passed_starting_point && candidate_and_focused_ordering == Ordering::Equal
                {
                    return (true, Continue::Yes);
                }
                // If the candidate does not have a lesser tab index, then discard it.
                if candidate_and_focused_ordering != Ordering::Less {
                    return (false, Continue::Yes);
                }
                let Some((_, winning_tab_index)) = self.current_winner else {
                    return (true, Continue::Yes);
                };
                // If the candidate element's tab index is not less than the current winner,
                // then it becomes the new winner. This means that when the tab indices are
                // equal, we give preference to the last one in DOM order.
                let should_select =
                    compare_tab_indices(candidate_element_tab_index, winning_tab_index) !=
                        Ordering::Less;
                (should_select, Continue::Yes)
            },
        }
    }
}

/// Compare two tab indices according to <https://html.spec.whatwg.org/multipage/#tabindex-value>.
///
/// `Ordering::Less`: The index should come before the other in sequential focus order.
/// `Ordering::Equal`: The two indices should be processed in DOM order, respecting focus direction
/// and focus scopes.
/// `Ordering::Greater`: The index should come after the other in sequential focus order.
///
/// Note that a tabindex of 0 should come after all others, which is essentially why we need this
/// function.
fn compare_tab_indices(a: i32, b: i32) -> Ordering {
    if a == b {
        Ordering::Equal
    } else if a == 0 {
        Ordering::Greater
    } else if b == 0 {
        Ordering::Less
    } else {
        a.cmp(&b)
    }
}
