/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use devtools_traits::{TimelineMarker, TimelineMarkerType};
use dom_struct::dom_struct;
use js::rust::HandleObject;
use metrics::ToMs;
use servo_atoms::Atom;

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding;
use crate::dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::Performance_Binding::PerformanceMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomObject, Reflector};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::{CompiledEventListener, EventTarget, ListenerPhase};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlinputelement::InputActivationState;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::performance::reduce_timing_resolution;
use crate::dom::virtualmethods::vtable_for;
use crate::dom::window::Window;
use crate::task::TaskOnce;

#[dom_struct]
pub struct Event {
    reflector_: Reflector,
    current_target: MutNullableDom<EventTarget>,
    target: MutNullableDom<EventTarget>,
    #[no_trace]
    type_: DomRefCell<Atom>,
    phase: Cell<EventPhase>,
    canceled: Cell<EventDefault>,
    stop_propagation: Cell<bool>,
    stop_immediate: Cell<bool>,
    cancelable: Cell<bool>,
    bubbles: Cell<bool>,
    trusted: Cell<bool>,
    dispatching: Cell<bool>,
    initialized: Cell<bool>,
    precise_time_ns: u64,
}

impl Event {
    pub fn new_inherited() -> Event {
        Event {
            reflector_: Reflector::new(),
            current_target: Default::default(),
            target: Default::default(),
            type_: DomRefCell::new(atom!("")),
            phase: Cell::new(EventPhase::None),
            canceled: Cell::new(EventDefault::Allowed),
            stop_propagation: Cell::new(false),
            stop_immediate: Cell::new(false),
            cancelable: Cell::new(false),
            bubbles: Cell::new(false),
            trusted: Cell::new(false),
            dispatching: Cell::new(false),
            initialized: Cell::new(false),
            precise_time_ns: time::precise_time_ns(),
        }
    }

    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<Event> {
        Self::new_uninitialized_with_proto(global, None)
    }

    pub fn new_uninitialized_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<Event> {
        reflect_dom_object_with_proto(Box::new(Event::new_inherited()), global, proto)
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) -> DomRoot<Event> {
        Self::new_with_proto(global, None, type_, bubbles, cancelable)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) -> DomRoot<Event> {
        let event = Event::new_uninitialized_with_proto(global, proto);
        event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        event
    }

    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        type_: DOMString,
        init: &EventBinding::EventInit,
    ) -> Fallible<DomRoot<Event>> {
        let bubbles = EventBubbles::from(init.bubbles);
        let cancelable = EventCancelable::from(init.cancelable);
        Ok(Event::new_with_proto(
            global,
            proto,
            Atom::from(type_),
            bubbles,
            cancelable,
        ))
    }

    pub fn init_event(&self, type_: Atom, bubbles: bool, cancelable: bool) {
        if self.dispatching.get() {
            return;
        }

        self.initialized.set(true);
        self.stop_propagation.set(false);
        self.stop_immediate.set(false);
        self.canceled.set(EventDefault::Allowed);
        self.trusted.set(false);
        self.target.set(None);
        *self.type_.borrow_mut() = type_;
        self.bubbles.set(bubbles);
        self.cancelable.set(cancelable);
    }

    /// <https://dom.spec.whatwg.org/#event-path>
    // TODO: shadow roots put special flags in the path,
    // and it will stop just being a list of bare EventTargets
    fn construct_event_path(&self, target: &EventTarget) -> Vec<DomRoot<EventTarget>> {
        let mut event_path = vec![];
        if let Some(target_node) = target.downcast::<Node>() {
            // ShadowIncluding::Yes might be closer to right than ::No,
            // but still wrong since things about the path change when crossing
            // shadow attachments; getting it right needs to change
            // more than just that.
            for ancestor in target_node.inclusive_ancestors(ShadowIncluding::No) {
                event_path.push(DomRoot::from_ref(ancestor.upcast::<EventTarget>()));
            }
            // Most event-target-to-parent relationships are node parent
            // relationships, but the document-to-global one is not,
            // so that's handled separately here.
            // (an EventTarget.get_parent_event_target could save
            // some redundancy, especially when shadow DOM relationships
            // also need to be respected)
            let top_most_ancestor_or_target = event_path
                .last()
                .cloned()
                .unwrap_or(DomRoot::from_ref(target));
            if let Some(document) = DomRoot::downcast::<Document>(top_most_ancestor_or_target) {
                if self.type_() != atom!("load") && document.browsing_context().is_some() {
                    event_path.push(DomRoot::from_ref(document.window().upcast()));
                }
            }
        } else {
            // a non-node EventTarget, likely a global.
            // No parent to propagate up to, but we still
            // need it on the path.
            event_path.push(DomRoot::from_ref(target));
        }
        event_path
    }

    /// <https://dom.spec.whatwg.org/#concept-event-dispatch>
    pub fn dispatch(
        &self,
        target: &EventTarget,
        legacy_target_override: bool,
        // TODO legacy_did_output_listeners_throw_flag for indexeddb
    ) -> EventStatus {
        // Step 1.
        self.dispatching.set(true);

        // Step 2.
        let target_override_document; // upcasted EventTarget's lifetime depends on this
        let target_override = if legacy_target_override {
            target_override_document = target
                .downcast::<Window>()
                .expect("legacy_target_override must be true only when target is a Window")
                .Document();
            target_override_document.upcast::<EventTarget>()
        } else {
            target
        };

        // Step 3 - since step 5 always happens, we can wait until 5.5

        // Step 4 TODO: "retargeting" concept depends on shadow DOM

        // Step 5, outer if-statement, is always true until step 4 is implemented
        // Steps 5.1-5.2 TODO: touch target lists don't exist yet

        // Steps 5.3 and most of 5.9
        // A change in whatwg/dom#240 specifies that
        // the event path belongs to the event itself, rather than being
        // a local variable of the dispatch algorithm, but this is mostly
        // related to shadow DOM requirements that aren't otherwise
        // implemented right now. The path also needs to contain
        // various flags instead of just bare event targets.
        let path = self.construct_event_path(target);
        rooted_vec!(let event_path <- path.into_iter());

        // Step 5.4
        let is_activation_event = self.is::<MouseEvent>() && self.type_() == atom!("click");

        // Step 5.5
        let mut activation_target = if is_activation_event {
            target
                .downcast::<Element>()
                .and_then(|e| e.as_maybe_activatable())
        } else {
            // Step 3
            None
        };

        // Steps 5-6 - 5.7 are shadow DOM slot things

        // Step 5.9.8.1, not covered in construct_event_path
        // This what makes sure that clicking on e.g. an <img> inside
        // an <a> will cause activation of the activatable ancestor.
        if is_activation_event && activation_target.is_none() && self.bubbles.get() {
            for object in event_path.iter() {
                if let Some(activatable_ancestor) = object
                    .downcast::<Element>()
                    .and_then(|e| e.as_maybe_activatable())
                {
                    activation_target = Some(activatable_ancestor);
                    // once activation_target isn't null, we stop
                    // looking at ancestors for it.
                    break;
                }
            }
        }

        // Steps 5.10-5.11 are shadow DOM

        // Not specified in dispatch spec overtly; this is because
        // the legacy canceled activation behavior of a checkbox
        // or radio button needs to know what happened in the
        // corresponding pre-activation behavior.
        let mut pre_activation_result: Option<InputActivationState> = None;

        // Step 5.12
        if is_activation_event {
            if let Some(maybe_checkbox) = activation_target {
                pre_activation_result = maybe_checkbox.legacy_pre_activation_behavior();
            }
        }

        let timeline_window = DomRoot::downcast::<Window>(target.global())
            .filter(|window| window.need_emit_timeline_marker(TimelineMarkerType::DOMEvent));

        // Step 5.13
        for object in event_path.iter().rev() {
            if &**object == target {
                self.phase.set(EventPhase::AtTarget);
            } else {
                self.phase.set(EventPhase::Capturing);
            }

            // setting self.target is step 1 of invoke,
            // but done here because our event_path isn't a member of self
            // (without shadow DOM, target_override is always the
            // target to set to)
            self.target.set(Some(target_override));
            invoke(
                timeline_window.as_deref(),
                object,
                self,
                Some(ListenerPhase::Capturing),
            );
        }

        // Step 5.14
        for object in event_path.iter() {
            let at_target = &**object == target;
            if at_target || self.bubbles.get() {
                self.phase.set(if at_target {
                    EventPhase::AtTarget
                } else {
                    EventPhase::Bubbling
                });

                self.target.set(Some(target_override));
                invoke(
                    timeline_window.as_deref(),
                    object,
                    self,
                    Some(ListenerPhase::Bubbling),
                );
            }
        }

        // Step 6
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

        // Step 7
        self.current_target.set(None);

        // Step 8 TODO: if path were in the event struct, we'd clear it now

        // Step 9
        self.dispatching.set(false);
        self.stop_propagation.set(false);
        self.stop_immediate.set(false);

        // Step 10 TODO: condition is always false until there's shadow DOM

        // Step 11
        if let Some(activation_target) = activation_target {
            if self.DefaultPrevented() {
                activation_target.legacy_canceled_activation_behavior(pre_activation_result);
            } else {
                activation_target.activation_behavior(self, target);
            }
        }

        self.status()
    }

    pub fn status(&self) -> EventStatus {
        if self.DefaultPrevented() {
            EventStatus::Canceled
        } else {
            EventStatus::NotCanceled
        }
    }

    #[inline]
    pub fn dispatching(&self) -> bool {
        self.dispatching.get()
    }

    #[inline]
    pub fn initialized(&self) -> bool {
        self.initialized.get()
    }

    #[inline]
    pub fn type_(&self) -> Atom {
        self.type_.borrow().clone()
    }

    #[inline]
    pub fn mark_as_handled(&self) {
        self.canceled.set(EventDefault::Handled);
    }

    #[inline]
    pub fn get_cancel_state(&self) -> EventDefault {
        self.canceled.get()
    }

    pub fn set_trusted(&self, trusted: bool) {
        self.trusted.set(trusted);
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-simple-event>
    pub fn fire(&self, target: &EventTarget) -> EventStatus {
        self.set_trusted(true);
        target.dispatch_event(self)
    }
}

impl EventMethods for Event {
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
        if let Some(target) = self.target.get() {
            self.construct_event_path(&target)
        } else {
            vec![]
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-event-defaultprevented>
    fn DefaultPrevented(&self) -> bool {
        self.canceled.get() == EventDefault::Prevented
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
        self.stop_immediate.set(true);
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
        reduce_timing_resolution(
            (self.precise_time_ns - (*self.global().performance().TimeOrigin()).round() as u64)
                .to_ms(),
        )
    }

    /// <https://dom.spec.whatwg.org/#dom-event-initevent>
    fn InitEvent(&self, type_: DOMString, bubbles: bool, cancelable: bool) {
        self.init_event(Atom::from(type_), bubbles, cancelable)
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.trusted.get()
    }
}

#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum EventBubbles {
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
pub enum EventCancelable {
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
pub enum EventPhase {
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
pub enum EventDefault {
    /// The default action of the event is allowed (constructor's default)
    Allowed,
    /// The default action has been prevented by calling `PreventDefault`
    Prevented,
    /// The event has been handled somewhere in the DOM, and it should be prevented from being
    /// re-handled elsewhere. This doesn't affect the judgement of `DefaultPrevented`
    Handled,
}

#[derive(PartialEq)]
pub enum EventStatus {
    Canceled,
    NotCanceled,
}

/// <https://dom.spec.whatwg.org/#concept-event-fire>
pub struct EventTask {
    pub target: Trusted<EventTarget>,
    pub name: Atom,
    pub bubbles: EventBubbles,
    pub cancelable: EventCancelable,
}

impl TaskOnce for EventTask {
    fn run_once(self) {
        let target = self.target.root();
        let bubbles = self.bubbles;
        let cancelable = self.cancelable;
        target.fire_event_with_params(self.name, bubbles, cancelable);
    }
}

/// <https://html.spec.whatwg.org/multipage/#fire-a-simple-event>
pub struct SimpleEventTask {
    pub target: Trusted<EventTarget>,
    pub name: Atom,
}

impl TaskOnce for SimpleEventTask {
    fn run_once(self) {
        let target = self.target.root();
        target.fire_event(self.name);
    }
}

/// <https://dom.spec.whatwg.org/#concept-event-listener-invoke>
fn invoke(
    timeline_window: Option<&Window>,
    object: &EventTarget,
    event: &Event,
    phase: Option<ListenerPhase>,
    // TODO legacy_output_did_listeners_throw for indexeddb
) {
    // Step 1: Until shadow DOM puts the event path in the
    // event itself, this is easier to do in dispatch before
    // calling invoke.

    // Step 2 TODO: relatedTarget only matters for shadow DOM

    // Step 3 TODO: touch target lists not implemented

    // Step 4.
    if event.stop_propagation.get() {
        return;
    }
    // Step 5.
    event.current_target.set(Some(object));

    // Step 6
    let listeners = object.get_listeners_for(&event.type_(), phase);

    // Step 7.
    let found = inner_invoke(timeline_window, object, event, &listeners);

    // Step 8
    if !found && event.trusted.get() {
        if let Some(legacy_type) = match event.type_() {
            atom!("animationend") => Some(atom!("webkitAnimationEnd")),
            atom!("animationiteration") => Some(atom!("webkitAnimationIteration")),
            atom!("animationstart") => Some(atom!("webkitAnimationStart")),
            atom!("transitionend") => Some(atom!("webkitTransitionEnd")),
            atom!("transitionrun") => Some(atom!("webkitTransitionRun")),
            _ => None,
        } {
            let original_type = event.type_();
            *event.type_.borrow_mut() = legacy_type;
            inner_invoke(timeline_window, object, event, &listeners);
            *event.type_.borrow_mut() = original_type;
        }
    }
}

/// <https://dom.spec.whatwg.org/#concept-event-listener-inner-invoke>
fn inner_invoke(
    timeline_window: Option<&Window>,
    object: &EventTarget,
    event: &Event,
    listeners: &[CompiledEventListener],
) -> bool {
    // Step 1.
    let mut found = false;

    // Step 2.
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

        // Step 2.2.
        found = true;

        // Step 2.5.
        if let CompiledEventListener::Listener(event_listener) = listener {
            object.remove_listener_if_once(&event.type_(), event_listener);
        }

        // Step 2.6
        let global = listener.associated_global();

        // Step 2.7-2.8
        let current_event = if let Some(window) = global.downcast::<Window>() {
            window.set_current_event(Some(event))
        } else {
            None
        };

        // Step 2.9 TODO: EventListener passive option not implemented

        // Step 2.10
        let marker = TimelineMarker::start("DOMEvent".to_owned());

        // Step 2.10
        listener.call_or_handle_event(object, event, ExceptionHandling::Report);

        if let Some(window) = timeline_window {
            window.emit_timeline_marker(marker.end());
        }

        // Step 2.11 TODO: passive not implemented

        // Step 2.12
        if let Some(window) = global.downcast::<Window>() {
            window.set_current_event(current_event.as_deref());
        }

        // Step 2.13: short-circuit instead of going to next listener
        if event.stop_immediate.get() {
            return found;
        }
    }

    // Step 3.
    found
}

impl Default for EventBinding::EventInit {
    fn default() -> Self {
        Self::empty()
    }
}
