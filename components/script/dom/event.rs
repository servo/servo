/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use base::cross_process_instant::CrossProcessInstant;
use devtools_traits::{TimelineMarker, TimelineMarkerType};
use dom_struct::dom_struct;
use js::rust::HandleObject;
use servo_atoms::Atom;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding;
use crate::dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMethods, ShadowRootMode,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::eventtarget::{CompiledEventListener, EventTarget, ListenerPhase};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlinputelement::InputActivationState;
use crate::dom::htmlslotelement::HTMLSlotElement;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;

/// <https://dom.spec.whatwg.org/#concept-event>
#[dom_struct]
pub(crate) struct Event {
    reflector_: Reflector,

    /// <https://dom.spec.whatwg.org/#dom-event-currenttarget>
    current_target: MutNullableDom<EventTarget>,

    /// <https://dom.spec.whatwg.org/#event-target>
    target: MutNullableDom<EventTarget>,

    /// <https://dom.spec.whatwg.org/#dom-event-type>
    #[no_trace]
    type_: DomRefCell<Atom>,

    /// <https://dom.spec.whatwg.org/#dom-event-eventphase>
    phase: Cell<EventPhase>,

    /// <https://dom.spec.whatwg.org/#canceled-flag>
    canceled: Cell<EventDefault>,

    /// <https://dom.spec.whatwg.org/#stop-propagation-flag>
    stop_propagation: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#stop-immediate-propagation-flag>
    stop_immediate_propagation: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dom-event-cancelable>
    cancelable: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dom-event-bubbles>
    bubbles: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dom-event-composed>
    composed: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    is_trusted: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dispatch-flag>
    dispatch: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#initialized-flag>
    initialized: Cell<bool>,

    /// <https://dom.spec.whatwg.org/#dom-event-timestamp>
    #[no_trace]
    time_stamp: CrossProcessInstant,

    /// <https://dom.spec.whatwg.org/#event-path>
    path: DomRefCell<Vec<EventPathSegment>>,

    /// <https://dom.spec.whatwg.org/#event-relatedtarget>
    related_target: MutNullableDom<EventTarget>,
}

/// An element on an [event path](https://dom.spec.whatwg.org/#event-path)
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct EventPathSegment {
    /// <https://dom.spec.whatwg.org/#event-path-invocation-target>
    invocation_target: Dom<EventTarget>,

    /// <https://dom.spec.whatwg.org/#event-path-invocation-target-in-shadow-tree>
    invocation_target_in_shadow_tree: bool,

    /// <https://dom.spec.whatwg.org/#event-path-shadow-adjusted-target>
    shadow_adjusted_target: Option<Dom<EventTarget>>,

    /// <https://dom.spec.whatwg.org/#event-path-relatedtarget>
    related_target: Option<Dom<EventTarget>>,

    /// <https://dom.spec.whatwg.org/#event-path-root-of-closed-tree>
    root_of_closed_tree: bool,

    /// <https://dom.spec.whatwg.org/#event-path-slot-in-closed-tree>
    slot_in_closed_tree: bool,
}

impl Event {
    pub(crate) fn new_inherited() -> Event {
        Event {
            reflector_: Reflector::new(),
            current_target: Default::default(),
            target: Default::default(),
            type_: DomRefCell::new(atom!("")),
            phase: Cell::new(EventPhase::None),
            canceled: Cell::new(EventDefault::Allowed),
            stop_propagation: Cell::new(false),
            stop_immediate_propagation: Cell::new(false),
            cancelable: Cell::new(false),
            bubbles: Cell::new(false),
            composed: Cell::new(false),
            is_trusted: Cell::new(false),
            dispatch: Cell::new(false),
            initialized: Cell::new(false),
            time_stamp: CrossProcessInstant::now(),
            path: DomRefCell::default(),
            related_target: Default::default(),
        }
    }

    pub(crate) fn new_uninitialized(global: &GlobalScope, can_gc: CanGc) -> DomRoot<Event> {
        Self::new_uninitialized_with_proto(global, None, can_gc)
    }

    pub(crate) fn new_uninitialized_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        reflect_dom_object_with_proto(Box::new(Event::new_inherited()), global, proto, can_gc)
    }

    pub(crate) fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable, can_gc)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        let event = Event::new_uninitialized_with_proto(global, proto, can_gc);

        // NOTE: The spec doesn't tell us to call init event here, it just happens to do what we need.
        event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        event
    }

    /// <https://dom.spec.whatwg.org/#dom-event-initevent>
    /// and <https://dom.spec.whatwg.org/#concept-event-initialize>
    pub(crate) fn init_event(&self, type_: Atom, bubbles: bool, cancelable: bool) {
        // https://dom.spec.whatwg.org/#dom-event-initevent
        if self.dispatch.get() {
            return;
        }

        // https://dom.spec.whatwg.org/#concept-event-initialize
        // Step 1. Set event’s initialized flag.
        self.initialized.set(true);

        // Step 2. Unset event’s stop propagation flag, stop immediate propagation flag, and canceled flag.
        self.stop_propagation.set(false);
        self.stop_immediate_propagation.set(false);
        self.canceled.set(EventDefault::Allowed);

        // Step 3. Set event’s isTrusted attribute to false.
        self.is_trusted.set(false);

        // Step 4. Set event’s target to null.
        self.target.set(None);

        // Step 5. Set event’s type attribute to type.
        *self.type_.borrow_mut() = type_;

        // Step 6. Set event’s bubbles attribute to bubbles.
        self.bubbles.set(bubbles);

        // Step 7. Set event’s cancelable attribute to cancelable.
        self.cancelable.set(cancelable);
    }

    pub(crate) fn set_target(&self, target_: Option<&EventTarget>) {
        self.target.set(target_);
    }

    /// <https://dom.spec.whatwg.org/#concept-event-path-append>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn append_to_path(
        &self,
        invocation_target: &EventTarget,
        shadow_adjusted_target: Option<&EventTarget>,
        related_target: Option<&EventTarget>,
        slot_in_closed_tree: bool,
    ) {
        // Step 1. Let invocationTargetInShadowTree be false.
        let mut invocation_target_in_shadow_tree = false;

        // Step 2. If invocationTarget is a node and its root is a shadow root,
        // then set invocationTargetInShadowTree to true.
        if invocation_target
            .downcast::<Node>()
            .is_some_and(Node::is_in_a_shadow_tree)
        {
            invocation_target_in_shadow_tree = true;
        }

        // Step 3. Let root-of-closed-tree be false.
        let mut root_of_closed_tree = false;

        // Step 4. If invocationTarget is a shadow root whose mode is "closed", then set root-of-closed-tree to true.
        if invocation_target
            .downcast::<ShadowRoot>()
            .is_some_and(|shadow_root| shadow_root.Mode() == ShadowRootMode::Closed)
        {
            root_of_closed_tree = true;
        }

        // Step 5. Append a new struct to event’s path whose invocation target is invocationTarget,
        // invocation-target-in-shadow-tree is invocationTargetInShadowTree, shadow-adjusted target is
        // shadowAdjustedTarget, relatedTarget is relatedTarget, touch target list is touchTargets,
        // root-of-closed-tree is root-of-closed-tree, and slot-in-closed-tree is slot-in-closed-tree.
        let event_path_segment = EventPathSegment {
            invocation_target: Dom::from_ref(invocation_target),
            shadow_adjusted_target: shadow_adjusted_target.map(Dom::from_ref),
            related_target: related_target.map(Dom::from_ref),
            invocation_target_in_shadow_tree,
            root_of_closed_tree,
            slot_in_closed_tree,
        };
        self.path.borrow_mut().push(event_path_segment);
    }

    /// <https://dom.spec.whatwg.org/#concept-event-dispatch>
    pub(crate) fn dispatch(
        &self,
        target: &EventTarget,
        legacy_target_override: bool,
        can_gc: CanGc,
        // TODO legacy_did_output_listeners_throw_flag for indexeddb
    ) -> EventStatus {
        let mut target = DomRoot::from_ref(target);

        // Step 1. Set event’s dispatch flag.
        self.dispatch.set(true);

        // Step 2. Let targetOverride be target, if legacy target override flag is not given,
        // and target’s associated Document otherwise.
        let target_override_document; // upcasted EventTarget's lifetime depends on this
        let target_override = if legacy_target_override {
            target_override_document = target
                .downcast::<Window>()
                .expect("legacy_target_override must be true only when target is a Window")
                .Document();
            DomRoot::from_ref(target_override_document.upcast::<EventTarget>())
        } else {
            target.clone()
        };

        // Step 3. Let activationTarget be null.
        let mut activation_target = None;

        // Step 4. Let relatedTarget be the result of retargeting event’s relatedTarget against target.
        let related_target = self
            .related_target
            .get()
            .map(|related_target| related_target.retarget(&target));

        // Step 5. If target is not relatedTarget or target is event’s relatedTarget:
        // Variables declared by the spec inside Step 5 but used later:
        // TODO: https://github.com/whatwg/dom/issues/1344
        let mut clear_targets = false;
        let mut pre_activation_result: Option<InputActivationState> = None;
        if related_target.as_ref() != Some(&target) ||
            self.related_target.get().as_ref() == Some(&target)
        {
            // TODO Step 5.1 Let touchTargets be a new list.
            // TODO Step 5.2 For each touchTarget of event’s touch target list, append the result of retargeting
            // touchTarget against target to touchTargets.

            // Step 5.3 Append to an event path with event, target, targetOverride, relatedTarget,
            // touchTargets, and false.
            self.append_to_path(
                &target,
                Some(target_override.upcast::<EventTarget>()),
                related_target.as_deref(),
                false,
            );

            // Step 5.4 Let isActivationEvent be true, if event is a MouseEvent object and
            // event’s type attribute is "click"; otherwise false.
            let is_activation_event = self.is::<MouseEvent>() && self.type_() == atom!("click");

            // Step 5.5 If isActivationEvent is true and target has activation behavior,
            // then set activationTarget to target.
            if is_activation_event {
                if let Some(element) = target.downcast::<Element>() {
                    if element.as_maybe_activatable().is_some() {
                        activation_target = Some(DomRoot::from_ref(element));
                    }
                }
            }

            // Step 5.6 Let slottable be target, if target is a slottable and is assigned, and null otherwise.
            let mut slottable = if target
                .downcast::<Node>()
                .and_then(Node::assigned_slot)
                .is_some()
            {
                Some(target.clone())
            } else {
                None
            };

            // Step 5.7 Let slot-in-closed-tree be false
            let mut slot_in_closed_tree = false;

            // Step 5.8 Let parent be the result of invoking target’s get the parent with event.
            let mut parent_or_none = target.get_the_parent(self);
            let mut done = false;

            // Step 5.9 While parent is non-null:
            while let Some(parent) = parent_or_none.clone() {
                // Step 5.9.1 If slottable is non-null:
                if slottable.is_some() {
                    // Step 5.9.1.1 Assert: parent is a slot.
                    let slot = parent
                        .downcast::<HTMLSlotElement>()
                        .expect("parent of slottable is not a slot");

                    // Step 5.9.1.2 Set slottable to null.
                    slottable = None;

                    // Step 5.9.1.3 If parent’s root is a shadow root whose mode is "closed",
                    // then set slot-in-closed-tree to true.
                    if slot
                        .containing_shadow_root()
                        .is_some_and(|root| root.Mode() == ShadowRootMode::Closed)
                    {
                        slot_in_closed_tree = true;
                    }
                }

                // Step 5.9.2 If parent is a slottable and is assigned, then set slottable to parent.
                if parent
                    .downcast::<Node>()
                    .and_then(Node::assigned_slot)
                    .is_some()
                {
                    slottable = Some(parent.clone());
                }

                // Step 5.9.3 Let relatedTarget be the result of retargeting event’s relatedTarget against parent.
                let related_target = self
                    .related_target
                    .get()
                    .map(|related_target| related_target.retarget(&target));

                // TODO: Step 5.9.4 Let touchTargets be a new list.
                // Step 5.9.5 For each touchTarget of event’s touch target list, append the result of retargeting
                // touchTarget against parent to touchTargets.

                // Step 5.9.6 If parent is a Window object, or parent is a node and target’s root is a
                // shadow-including inclusive ancestor of parent:
                let root_is_shadow_inclusive_ancestor = parent
                    .downcast::<Node>()
                    .zip(target.downcast::<Node>())
                    .is_some_and(|(parent, target)| {
                        target
                            .GetRootNode(&GetRootNodeOptions::empty())
                            .is_shadow_including_inclusive_ancestor_of(parent)
                    });
                if parent.is::<Window>() || root_is_shadow_inclusive_ancestor {
                    // Step 5.9.6.1 If isActivationEvent is true, event’s bubbles attribute is true, activationTarget
                    // is null, and parent has activation behavior, then set activationTarget to parent.
                    if is_activation_event && activation_target.is_none() && self.bubbles.get() {
                        if let Some(element) = parent.downcast::<Element>() {
                            if element.as_maybe_activatable().is_some() {
                                activation_target = Some(DomRoot::from_ref(element));
                            }
                        }
                    }

                    // Step 5.9.6.2 Append to an event path with event, parent, null, relatedTarget, touchTargets,
                    // and slot-in-closed-tree.
                    self.append_to_path(
                        &parent,
                        None,
                        related_target.as_deref(),
                        slot_in_closed_tree,
                    );
                }
                // Step 5.9.7 Otherwise, if parent is relatedTarget, then set parent to null.
                else if Some(&parent) == related_target.as_ref() {
                    // NOTE: This causes some lifetime shenanigans. Instead of making things complicated,
                    // we just remember to treat parent as null later
                    done = true;
                }
                // Step 5.9.8 Otherwise:
                else {
                    // Step 5.9.8.1 Set target to parent.
                    target = parent.clone();

                    // Step 5.9.8.2 If isActivationEvent is true, activationTarget is null, and target has
                    // activation behavior, then set activationTarget to target.
                    if is_activation_event && activation_target.is_none() {
                        if let Some(element) = parent.downcast::<Element>() {
                            if element.as_maybe_activatable().is_some() {
                                activation_target = Some(DomRoot::from_ref(element));
                            }
                        }
                    }

                    // Step 5.9.8.3 Append to an event path with event, parent, target, relatedTarget,
                    // touchTargets, and slot-in-closed-tree.
                    self.append_to_path(
                        &parent,
                        Some(&target),
                        related_target.as_deref(),
                        slot_in_closed_tree,
                    );
                }

                // Step 5.9.9 If parent is non-null, then set parent to the result of invoking parent’s
                // get the parent with event
                if !done {
                    parent_or_none = parent.get_the_parent(self);
                }

                // Step 5.9.10 Set slot-in-closed-tree to false.
                slot_in_closed_tree = false;
            }

            // Step 5.10 Let clearTargetsStruct be the last struct in event’s path whose shadow-adjusted target
            // is non-null.
            // Step 5.11 Let clearTargets be true if clearTargetsStruct’s shadow-adjusted target,
            // clearTargetsStruct’s relatedTarget, or an EventTarget object in clearTargetsStruct’s
            // touch target list is a node and its root is a shadow root; otherwise false.
            // TODO: Handle touch target list
            clear_targets = self
                .path
                .borrow()
                .iter()
                .rev()
                .find(|segment| segment.shadow_adjusted_target.is_some())
                // This is "clearTargetsStruct"
                .is_some_and(|clear_targets| {
                    clear_targets
                        .shadow_adjusted_target
                        .as_ref()
                        .and_then(|target| target.downcast::<Node>())
                        .is_some_and(Node::is_in_a_shadow_tree) ||
                        clear_targets
                            .related_target
                            .as_ref()
                            .and_then(|target| target.downcast::<Node>())
                            .is_some_and(Node::is_in_a_shadow_tree)
                });

            // Step 5.12 If activationTarget is non-null and activationTarget has legacy-pre-activation behavior,
            // then run activationTarget’s legacy-pre-activation behavior.
            if let Some(activation_target) = activation_target.as_ref() {
                // Not specified in dispatch spec overtly; this is because
                // the legacy canceled activation behavior of a checkbox
                // or radio button needs to know what happened in the
                // corresponding pre-activation behavior.
                pre_activation_result = activation_target
                    .as_maybe_activatable()
                    .and_then(|activatable| activatable.legacy_pre_activation_behavior());
            }

            let timeline_window = DomRoot::downcast::<Window>(target.global())
                .filter(|window| window.need_emit_timeline_marker(TimelineMarkerType::DOMEvent));

            // Step 5.13 For each struct in event’s path, in reverse order:
            for (index, segment) in self.path.borrow().iter().enumerate().rev() {
                // Step 5.13.1 If struct’s shadow-adjusted target is non-null, then set event’s
                // eventPhase attribute to AT_TARGET.
                if segment.shadow_adjusted_target.is_some() {
                    self.phase.set(EventPhase::AtTarget);
                }
                // Step 5.13.2 Otherwise, set event’s eventPhase attribute to CAPTURING_PHASE.
                else {
                    self.phase.set(EventPhase::Capturing);
                }

                // Step 5.13.3 Invoke with struct, event, "capturing", and legacyOutputDidListenersThrowFlag if given.
                invoke(
                    segment,
                    index,
                    self,
                    ListenerPhase::Capturing,
                    timeline_window.as_deref(),
                    can_gc,
                )
            }

            // Step 5.14 For each struct in event’s path:
            for (index, segment) in self.path.borrow().iter().enumerate() {
                // Step 5.14.1 If struct’s shadow-adjusted target is non-null, then set event’s
                // eventPhase attribute to AT_TARGET.
                if segment.shadow_adjusted_target.is_some() {
                    self.phase.set(EventPhase::AtTarget);
                }
                // Step 5.14.2 Otherwise:
                else {
                    // Step 5.14.2.1 If event’s bubbles attribute is false, then continue.
                    if !self.bubbles.get() {
                        continue;
                    }

                    // Step 5.14.2.2 Set event’s eventPhase attribute to BUBBLING_PHASE.
                    self.phase.set(EventPhase::Bubbling);
                }

                // Step 5.14.3 Invoke with struct, event, "bubbling", and legacyOutputDidListenersThrowFlag if given.
                invoke(
                    segment,
                    index,
                    self,
                    ListenerPhase::Bubbling,
                    timeline_window.as_deref(),
                    can_gc,
                );
            }
        }

        // Step 6. Set event’s eventPhase attribute to NONE.
        self.phase.set(EventPhase::None);

        // FIXME: The UIEvents spec still expects firing an event
        // to carry a "default action" semantic, but the HTML spec
        // has removed this concept. Nothing in either spec currently
        // (as of Jan 11 2020) says that, e.g., a keydown event on an
        // input element causes a character to be typed; the UIEvents
        // spec assumes the HTML spec is covering it, and the HTML spec
        // no longer specifies any UI event other than mouse click as
        // causing an element to perform an action.
        // Compare:
        // https://w3c.github.io/uievents/#default-action
        // https://dom.spec.whatwg.org/#action-versus-occurance
        if !self.DefaultPrevented() {
            if let Some(target) = self.GetTarget() {
                if let Some(node) = target.downcast::<Node>() {
                    let vtable = vtable_for(node);
                    vtable.handle_event(self);
                }
            }
        }

        // Step 7. Set event’s currentTarget attribute to null.
        self.current_target.set(None);

        // Step 8. Set event’s path to the empty list.
        self.path.borrow_mut().clear();

        // Step 9. Unset event’s dispatch flag, stop propagation flag, and stop immediate propagation flag.
        self.dispatch.set(false);
        self.stop_propagation.set(false);
        self.stop_immediate_propagation.set(false);

        // Step 10. If clearTargets is true:
        if clear_targets {
            // Step 10.1 Set event’s target to null.
            self.target.set(None);

            // Step 10.2 Set event’s relatedTarget to null.
            self.related_target.set(None);

            // TODO Step 10.3 Set event’s touch target list to the empty list.
        }

        // Step 11. If activationTarget is non-null:
        if let Some(activation_target) = activation_target {
            // NOTE: The activation target may have been disabled by an event handler
            if let Some(activatable) = activation_target.as_maybe_activatable() {
                // Step 11.1 If event’s canceled flag is unset, then run activationTarget’s
                // activation behavior with event.
                if !self.DefaultPrevented() {
                    activatable.activation_behavior(self, &target, can_gc);
                }
                // Step 11.2 Otherwise, if activationTarget has legacy-canceled-activation behavior, then run
                // activationTarget’s legacy-canceled-activation behavior.
                else {
                    activatable.legacy_canceled_activation_behavior(pre_activation_result);
                }
            }
        }

        // Step 12 Return false if event’s canceled flag is set; otherwise true.
        self.status()
    }

    pub(crate) fn status(&self) -> EventStatus {
        if self.DefaultPrevented() {
            EventStatus::Canceled
        } else {
            EventStatus::NotCanceled
        }
    }

    #[inline]
    pub(crate) fn dispatching(&self) -> bool {
        self.dispatch.get()
    }

    #[inline]
    pub(crate) fn initialized(&self) -> bool {
        self.initialized.get()
    }

    #[inline]
    pub(crate) fn type_(&self) -> Atom {
        self.type_.borrow().clone()
    }

    #[inline]
    pub(crate) fn mark_as_handled(&self) {
        self.canceled.set(EventDefault::Handled);
    }

    #[inline]
    pub(crate) fn get_cancel_state(&self) -> EventDefault {
        self.canceled.get()
    }

    pub(crate) fn set_trusted(&self, trusted: bool) {
        self.is_trusted.set(trusted);
    }

    pub(crate) fn set_composed(&self, composed: bool) {
        self.composed.set(composed);
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-simple-event>
    pub(crate) fn fire(&self, target: &EventTarget, can_gc: CanGc) -> EventStatus {
        self.set_trusted(true);
        target.dispatch_event(self, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#inner-event-creation-steps>
    fn inner_creation_steps(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        init: &EventBinding::EventInit,
        can_gc: CanGc,
    ) -> DomRoot<Event> {
        // Step 1. Let event be the result of creating a new object using eventInterface.
        // If realm is non-null, then use that realm; otherwise, use the default behavior defined in Web IDL.
        let event = Event::new_uninitialized_with_proto(global, proto, can_gc);

        // Step 2. Set event’s initialized flag.
        event.initialized.set(true);

        // Step 3. Initialize event’s timeStamp attribute to the relative high resolution
        // coarse time given time and event’s relevant global object.
        // NOTE: This is done inside Event::new_inherited

        // Step 3. For each member → value in dictionary, if event has an attribute whose
        // identifier is member, then initialize that attribute to value.#
        event.bubbles.set(init.bubbles);
        event.cancelable.set(init.cancelable);
        event.composed.set(init.composed);

        // Step 5. Run the event constructing steps with event and dictionary.
        // NOTE: Event construction steps may be defined by subclasses

        // Step 6. Return event.
        event
    }

    /// Implements the logic behind the [get the parent](https://dom.spec.whatwg.org/#get-the-parent)
    /// algorithm for shadow roots.
    pub(crate) fn should_pass_shadow_boundary(&self, shadow_root: &ShadowRoot) -> bool {
        debug_assert!(self.dispatching());

        // > A shadow root’s get the parent algorithm, given an event, returns null if event’s composed flag
        // > is unset and shadow root is the root of event’s path’s first struct’s invocation target;
        // > otherwise shadow root’s host.
        if self.Composed() {
            return true;
        }

        let path = self.path.borrow();
        let first_invocation_target = &path
            .first()
            .expect("Event path is empty despite event currently being dispatched")
            .invocation_target
            .as_rooted();

        // The spec doesn't tell us what should happen if the invocation target is not a node
        let Some(target_node) = first_invocation_target.downcast::<Node>() else {
            return false;
        };

        &*target_node.GetRootNode(&GetRootNodeOptions::empty()) != shadow_root.upcast::<Node>()
    }
}

impl EventMethods<crate::DomTypeHolder> for Event {
    /// <https://dom.spec.whatwg.org/#concept-event-constructor>
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        type_: DOMString,
        init: &EventBinding::EventInit,
    ) -> Fallible<DomRoot<Event>> {
        // Step 1. Let event be the result of running the inner event creation steps with
        // this interface, null, now, and eventInitDict.
        let event = Event::inner_creation_steps(global, proto, init, can_gc);

        // Step 2. Initialize event’s type attribute to type.
        *event.type_.borrow_mut() = Atom::from(type_);

        // Step 3. Return event.
        Ok(event)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-eventphase>
    fn EventPhase(&self) -> u16 {
        self.phase.get() as u16
    }

    /// <https://dom.spec.whatwg.org/#dom-event-type>
    fn Type(&self) -> DOMString {
        DOMString::from(&*self.type_()) // FIXME(ajeffrey): Directly convert from Atom to DOMString
    }

    /// <https://dom.spec.whatwg.org/#dom-event-target>
    fn GetTarget(&self) -> Option<DomRoot<EventTarget>> {
        self.target.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-srcelement>
    fn GetSrcElement(&self) -> Option<DomRoot<EventTarget>> {
        self.target.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-currenttarget>
    fn GetCurrentTarget(&self) -> Option<DomRoot<EventTarget>> {
        self.current_target.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-composedpath>
    fn ComposedPath(&self) -> Vec<DomRoot<EventTarget>> {
        // Step 1. Let composedPath be an empty list.
        let mut composed_path = vec![];

        // Step 2. Let path be this’s path.
        let path = self.path.borrow();

        // Step 3. If path is empty, then return composedPath.
        if path.is_empty() {
            return composed_path;
        }

        // Step 4. Let currentTarget be this’s currentTarget attribute value.
        let current_target = self.GetCurrentTarget();

        // Step 5. Append currentTarget to composedPath.
        // TODO: https://github.com/whatwg/dom/issues/1343
        composed_path.push(current_target.clone().expect(
            "Since the event's path is not empty it is being dispatched and must have a current target",
        ));

        // Step 6. Let currentTargetIndex be 0.
        let mut current_target_index = 0;

        // Step 7. Let currentTargetHiddenSubtreeLevel be 0.
        let mut current_target_hidden_subtree_level = 0;

        // Step 8. Let index be path’s size − 1.
        // Step 9. While index is greater than or equal to 0:
        // NOTE: This is just iterating the path in reverse
        for (index, element) in path.iter().enumerate().rev() {
            // Step 9.1 If path[index]'s root-of-closed-tree is true, then increase
            // currentTargetHiddenSubtreeLevel by 1.
            if element.root_of_closed_tree {
                current_target_hidden_subtree_level += 1;
            }

            // Step 9.2 If path[index]'s invocation target is currentTarget, then set
            // currentTargetIndex to index and break.
            if current_target
                .as_ref()
                .is_some_and(|target| target.as_traced() == element.invocation_target)
            {
                current_target_index = index;
                break;
            }

            // Step 9.3 If path[index]'s slot-in-closed-tree is true, then decrease
            // currentTargetHiddenSubtreeLevel by 1.
            if element.slot_in_closed_tree {
                current_target_hidden_subtree_level -= 1;
            }

            // Step 9.4 Decrease index by 1.
        }

        // Step 10. Let currentHiddenLevel and maxHiddenLevel be currentTargetHiddenSubtreeLevel.
        let mut current_hidden_level = current_target_hidden_subtree_level;
        let mut max_hidden_level = current_target_hidden_subtree_level;

        // Step 11. Set index to currentTargetIndex − 1.
        // Step 12. While index is greater than or equal to 0:
        // NOTE: This is just iterating part of the path in reverse
        for element in path.iter().take(current_target_index).rev() {
            // Step 12.1 If path[index]'s root-of-closed-tree is true, then increase currentHiddenLevel by 1.
            if element.root_of_closed_tree {
                current_hidden_level += 1;
            }

            // Step 12.2 If currentHiddenLevel is less than or equal to maxHiddenLevel,
            // then prepend path[index]'s invocation target to composedPath.
            if current_hidden_level <= max_hidden_level {
                composed_path.insert(0, element.invocation_target.as_rooted());
            }

            // Step 12.3 If path[index]'s slot-in-closed-tree is true:
            if element.slot_in_closed_tree {
                // Step 12.3.1 Decrease currentHiddenLevel by 1.
                current_hidden_level -= 1;

                // Step 12.3.2 If currentHiddenLevel is less than maxHiddenLevel, then set
                // maxHiddenLevel to currentHiddenLevel.
                if current_hidden_level < max_hidden_level {
                    max_hidden_level = current_hidden_level;
                }
            }

            // Step 12.4 Decrease index by 1.
        }

        // Step 13. Set currentHiddenLevel and maxHiddenLevel to currentTargetHiddenSubtreeLevel.
        current_hidden_level = current_target_hidden_subtree_level;
        max_hidden_level = current_target_hidden_subtree_level;

        // Step 14. Set index to currentTargetIndex + 1.
        // Step 15. While index is less than path’s size:
        // NOTE: This is just iterating the list and skipping the first current_target_index + 1 elements
        //       (The +1 is necessary because the index is 0-based and the skip method is not)
        for element in path.iter().skip(current_target_index + 1) {
            // Step 15.1 If path[index]'s slot-in-closed-tree is true, then increase currentHiddenLevel by 1.
            if element.slot_in_closed_tree {
                current_hidden_level += 1;
            }

            // Step 15.2 If currentHiddenLevel is less than or equal to maxHiddenLevel,
            // then append path[index]'s invocation target to composedPath.
            if current_hidden_level <= max_hidden_level {
                composed_path.push(element.invocation_target.as_rooted());
            }

            // Step 15.3 If path[index]'s root-of-closed-tree is true:
            if element.root_of_closed_tree {
                // Step 15.3.1 Decrease currentHiddenLevel by 1.
                current_hidden_level -= 1;

                // Step 15.3.2 If currentHiddenLevel is less than maxHiddenLevel, then set
                // maxHiddenLevel to currentHiddenLevel.
                if current_hidden_level < max_hidden_level {
                    max_hidden_level = current_hidden_level;
                }
            }

            // Step 15.4 Increase index by 1.
        }

        // Step 16. Return composedPath.
        composed_path
    }

    /// <https://dom.spec.whatwg.org/#dom-event-defaultprevented>
    fn DefaultPrevented(&self) -> bool {
        self.canceled.get() == EventDefault::Prevented
    }

    /// <https://dom.spec.whatwg.org/#dom-event-composed>
    fn Composed(&self) -> bool {
        self.composed.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-preventdefault>
    fn PreventDefault(&self) {
        if self.cancelable.get() {
            self.canceled.set(EventDefault::Prevented)
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-event-stoppropagation>
    fn StopPropagation(&self) {
        self.stop_propagation.set(true);
    }

    /// <https://dom.spec.whatwg.org/#dom-event-stopimmediatepropagation>
    fn StopImmediatePropagation(&self) {
        self.stop_immediate_propagation.set(true);
        self.stop_propagation.set(true);
    }

    /// <https://dom.spec.whatwg.org/#dom-event-bubbles>
    fn Bubbles(&self) -> bool {
        self.bubbles.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-cancelable>
    fn Cancelable(&self) -> bool {
        self.cancelable.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-returnvalue>
    fn ReturnValue(&self) -> bool {
        self.canceled.get() == EventDefault::Allowed
    }

    /// <https://dom.spec.whatwg.org/#dom-event-returnvalue>
    fn SetReturnValue(&self, val: bool) {
        if !val {
            self.PreventDefault();
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-event-cancelbubble>
    fn CancelBubble(&self) -> bool {
        self.stop_propagation.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-cancelbubble>
    fn SetCancelBubble(&self, value: bool) {
        if value {
            self.stop_propagation.set(true)
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-event-timestamp>
    fn TimeStamp(&self) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .to_dom_high_res_time_stamp(self.time_stamp)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-initevent>
    fn InitEvent(&self, type_: DOMString, bubbles: bool, cancelable: bool) {
        self.init_event(Atom::from(type_), bubbles, cancelable)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.is_trusted.get()
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum EventBubbles {
    Bubbles,
    DoesNotBubble,
}

impl From<bool> for EventBubbles {
    fn from(boolean: bool) -> Self {
        if boolean {
            EventBubbles::Bubbles
        } else {
            EventBubbles::DoesNotBubble
        }
    }
}

impl From<EventBubbles> for bool {
    fn from(bubbles: EventBubbles) -> Self {
        match bubbles {
            EventBubbles::Bubbles => true,
            EventBubbles::DoesNotBubble => false,
        }
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum EventCancelable {
    Cancelable,
    NotCancelable,
}

impl From<bool> for EventCancelable {
    fn from(boolean: bool) -> Self {
        if boolean {
            EventCancelable::Cancelable
        } else {
            EventCancelable::NotCancelable
        }
    }
}

impl From<EventCancelable> for bool {
    fn from(bubbles: EventCancelable) -> Self {
        match bubbles {
            EventCancelable::Cancelable => true,
            EventCancelable::NotCancelable => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, JSTraceable, PartialEq)]
#[repr(u16)]
#[derive(MallocSizeOf)]
pub(crate) enum EventPhase {
    None = EventConstants::NONE,
    Capturing = EventConstants::CAPTURING_PHASE,
    AtTarget = EventConstants::AT_TARGET,
    Bubbling = EventConstants::BUBBLING_PHASE,
}

/// An enum to indicate whether the default action of an event is allowed.
///
/// This should've been a bool. Instead, it's an enum, because, aside from the allowed/canceled
/// states, we also need something to stop the event from being handled again (without cancelling
/// the event entirely). For example, an Up/Down `KeyEvent` inside a `textarea` element will
/// trigger the cursor to go up/down if the text inside the element spans multiple lines. This enum
/// helps us to prevent such events from being [sent to the constellation][msg] where it will be
/// handled once again for page scrolling (which is definitely not what we'd want).
///
/// [msg]: https://doc.servo.org/compositing/enum.ConstellationMsg.html#variant.KeyEvent
///
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum EventDefault {
    /// The default action of the event is allowed (constructor's default)
    Allowed,
    /// The default action has been prevented by calling `PreventDefault`
    Prevented,
    /// The event has been handled somewhere in the DOM, and it should be prevented from being
    /// re-handled elsewhere. This doesn't affect the judgement of `DefaultPrevented`
    Handled,
}

#[derive(PartialEq)]
pub(crate) enum EventStatus {
    Canceled,
    NotCanceled,
}

/// <https://dom.spec.whatwg.org/#concept-event-fire>
pub(crate) struct EventTask {
    pub(crate) target: Trusted<EventTarget>,
    pub(crate) name: Atom,
    pub(crate) bubbles: EventBubbles,
    pub(crate) cancelable: EventCancelable,
}

impl TaskOnce for EventTask {
    fn run_once(self) {
        let target = self.target.root();
        let bubbles = self.bubbles;
        let cancelable = self.cancelable;
        target.fire_event_with_params(self.name, bubbles, cancelable, CanGc::note());
    }
}

/// <https://html.spec.whatwg.org/multipage/#fire-a-simple-event>
pub(crate) struct SimpleEventTask {
    pub(crate) target: Trusted<EventTarget>,
    pub(crate) name: Atom,
}

impl TaskOnce for SimpleEventTask {
    fn run_once(self) {
        let target = self.target.root();
        target.fire_event(self.name, CanGc::note());
    }
}

/// <https://dom.spec.whatwg.org/#concept-event-listener-invoke>
fn invoke(
    segment: &EventPathSegment,
    segment_index_in_path: usize,
    event: &Event,
    phase: ListenerPhase,
    timeline_window: Option<&Window>,
    can_gc: CanGc,
    // TODO legacy_output_did_listeners_throw for indexeddb
) {
    // Step 1. Set event’s target to the shadow-adjusted target of the last struct in event’s path,
    // that is either struct or preceding struct, whose shadow-adjusted target is non-null.
    event.target.set(
        event.path.borrow()[..segment_index_in_path + 1]
            .iter()
            .rev()
            .flat_map(|segment| segment.shadow_adjusted_target.clone())
            .next()
            .as_deref(),
    );

    // Step 2. Set event’s relatedTarget to struct’s relatedTarget.
    event.related_target.set(segment.related_target.as_deref());

    // TODO: Set event’s touch target list to struct’s touch target list.

    // Step 4. If event’s stop propagation flag is set, then return.
    if event.stop_propagation.get() {
        return;
    }

    // Step 5. Initialize event’s currentTarget attribute to struct’s invocation target.
    event.current_target.set(Some(&segment.invocation_target));

    // Step 6. Let listeners be a clone of event’s currentTarget attribute value’s event listener list.
    let listeners =
        segment
            .invocation_target
            .get_listeners_for(&event.type_(), Some(phase), can_gc);

    // Step 7. Let invocationTargetInShadowTree be struct’s invocation-target-in-shadow-tree.
    let invocation_target_in_shadow_tree = segment.invocation_target_in_shadow_tree;

    // Step 8. Let found be the result of running inner invoke with event, listeners, phase,
    // invocationTargetInShadowTree, and legacyOutputDidListenersThrowFlag if given.
    let found = inner_invoke(
        event,
        &listeners,
        phase,
        invocation_target_in_shadow_tree,
        timeline_window,
    );

    // Step 9. If found is false and event’s isTrusted attribute is true:
    if !found && event.is_trusted.get() {
        // Step 9.1 Let originalEventType be event’s type attribute value.
        let original_type = event.type_();

        // Step 9.2 If event’s type attribute value is a match for any of the strings in the first column
        // in the following table, set event’s type attribute value to the string in the second column on
        // the same row as the matching string, and return otherwise.
        let legacy_type = match event.type_() {
            atom!("animationend") => atom!("webkitAnimationEnd"),
            atom!("animationiteration") => atom!("webkitAnimationIteration"),
            atom!("animationstart") => atom!("webkitAnimationStart"),
            atom!("transitionend") => atom!("webkitTransitionEnd"),
            atom!("transitionrun") => atom!("webkitTransitionRun"),
            _ => return,
        };
        *event.type_.borrow_mut() = legacy_type;

        // Step 9.3 Inner invoke with event, listeners, phase, invocationTargetInShadowTree,
        // and legacyOutputDidListenersThrowFlag if given.
        inner_invoke(
            event,
            &listeners,
            phase,
            invocation_target_in_shadow_tree,
            timeline_window,
        );

        // Step 9.4 Set event’s type attribute value to originalEventType.
        *event.type_.borrow_mut() = original_type;
    }
}

/// <https://dom.spec.whatwg.org/#concept-event-listener-inner-invoke>
fn inner_invoke(
    event: &Event,
    listeners: &[CompiledEventListener],
    _phase: ListenerPhase,
    invocation_target_in_shadow_tree: bool,
    timeline_window: Option<&Window>,
) -> bool {
    // Step 1. Let found be false.
    let mut found = false;

    // Step 2. For each listener in listeners, whose removed is false:
    for listener in listeners {
        // FIXME(#25479): We need an "if !listener.removed()" here,
        // but there's a subtlety. Where Servo is currently using the
        // CompiledEventListener, we really need something that maps to
        // https://dom.spec.whatwg.org/#concept-event-listener
        // which is not the same thing as the EventListener interface.
        // script::dom::eventtarget::EventListenerEntry is the closest
        // match we have, and is already holding the "once" flag,
        // but it's not a drop-in replacement.

        // Steps 2.1 and 2.3-2.4 are not done because `listeners` contain only the
        // relevant ones for this invoke call during the dispatch algorithm.
        // TODO: Step 2.1 If event’s type attribute value is not listener’s type, then continue.

        // Step 2.2. Set found to true.
        found = true;

        // TODO Step 2.3 If phase is "capturing" and listener’s capture is false, then continue.
        // TODO Step 2.4 If phase is "bubbling" and listener’s capture is true, then continue.

        // Step 2.5 If listener’s once is true, then remove an event listener given event’s currentTarget
        // attribute value and listener.
        if let CompiledEventListener::Listener(event_listener) = listener {
            event
                .GetCurrentTarget()
                .expect("event target was initialized as part of \"invoke\"")
                .remove_listener_if_once(&event.type_(), event_listener);
        }

        // Step 2.6 Let global be listener callback’s associated realm’s global object.
        let global = listener.associated_global();

        // Step 2.7 Let currentEvent be undefined.
        let mut current_event = None;
        // Step 2.8 If global is a Window object:
        if let Some(window) = global.downcast::<Window>() {
            // Step 2.8.1 Set currentEvent to global’s current event.
            current_event = window.current_event();

            // Step 2.8.2 If invocationTargetInShadowTree is false, then set global’s current event to event.
            if !invocation_target_in_shadow_tree {
                current_event = window.set_current_event(Some(event))
            }
        }

        // TODO Step 2.9 If listener’s passive is true, then set event’s in passive listener flag.

        // Step 2.10 If global is a Window object, then record timing info for event listener given event and listener.
        // Step 2.11 Call a user object’s operation with listener’s callback, "handleEvent", « event »,
        // and event’s currentTarget attribute value. If this throws an exception exception:
        //     Step 2.10.1 Report exception for listener’s callback’s corresponding JavaScript object’s
        //     associated realm’s global object.
        //     TODO Step 2.10.2 Set legacyOutputDidListenersThrowFlag if given.
        let marker = TimelineMarker::start("DOMEvent".to_owned());
        listener.call_or_handle_event(
            &event
                .GetCurrentTarget()
                .expect("event target was initialized as part of \"invoke\""),
            event,
            ExceptionHandling::Report,
        );
        if let Some(window) = timeline_window {
            window.emit_timeline_marker(marker.end());
        }

        // TODO Step 2.12 Unset event’s in passive listener flag.

        // Step 2.13 If global is a Window object, then set global’s current event to currentEvent.
        if let Some(window) = global.downcast::<Window>() {
            window.set_current_event(current_event.as_deref());
        }

        // Step 2.13: If event’s stop immediate propagation flag is set, then break.
        if event.stop_immediate_propagation.get() {
            break;
        }
    }

    // Step 3.
    found
}
