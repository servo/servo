/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{TimelineMarker, TimelineMarkerType};
use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding;
use dom::bindings::codegen::Bindings::EventBinding::{EventConstants, EventMethods};
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::eventtarget::{CompiledEventListener, EventTarget, ListenerPhase};
use dom::globalscope::GlobalScope;
use dom::node::Node;
use dom::virtualmethods::vtable_for;
use dom::window::Window;
use dom_struct::dom_struct;
use script_thread::Runnable;
use servo_atoms::Atom;
use std::cell::Cell;
use std::default::Default;
use time;

#[dom_struct]
pub struct Event {
    reflector_: Reflector,
    current_target: MutNullableJS<EventTarget>,
    target: MutNullableJS<EventTarget>,
    type_: DOMRefCell<Atom>,
    phase: Cell<EventPhase>,
    canceled: Cell<EventDefault>,
    stop_propagation: Cell<bool>,
    stop_immediate: Cell<bool>,
    cancelable: Cell<bool>,
    bubbles: Cell<bool>,
    trusted: Cell<bool>,
    dispatching: Cell<bool>,
    initialized: Cell<bool>,
    timestamp: u64,
}

impl Event {
    pub fn new_inherited() -> Event {
        Event {
            reflector_: Reflector::new(),
            current_target: Default::default(),
            target: Default::default(),
            type_: DOMRefCell::new(atom!("")),
            phase: Cell::new(EventPhase::None),
            canceled: Cell::new(EventDefault::Allowed),
            stop_propagation: Cell::new(false),
            stop_immediate: Cell::new(false),
            cancelable: Cell::new(false),
            bubbles: Cell::new(false),
            trusted: Cell::new(false),
            dispatching: Cell::new(false),
            initialized: Cell::new(false),
            timestamp: time::get_time().sec as u64,
        }
    }

    pub fn new_uninitialized(global: &GlobalScope) -> Root<Event> {
        reflect_dom_object(box Event::new_inherited(),
                           global,
                           EventBinding::Wrap)
    }

    pub fn new(global: &GlobalScope,
               type_: Atom,
               bubbles: EventBubbles,
               cancelable: EventCancelable) -> Root<Event> {
        let event = Event::new_uninitialized(global);
        event.init_event(type_, bool::from(bubbles), bool::from(cancelable));
        event
    }

    pub fn Constructor(global: &GlobalScope,
                       type_: DOMString,
                       init: &EventBinding::EventInit) -> Fallible<Root<Event>> {
        let bubbles = EventBubbles::from(init.bubbles);
        let cancelable = EventCancelable::from(init.cancelable);
        Ok(Event::new(global, Atom::from(type_), bubbles, cancelable))
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

    // https://dom.spec.whatwg.org/#concept-event-dispatch
    pub fn dispatch(&self,
                    target: &EventTarget,
                    target_override: Option<&EventTarget>)
                    -> EventStatus {
        assert!(!self.dispatching());
        assert!(self.initialized());
        assert_eq!(self.phase.get(), EventPhase::None);
        assert!(self.GetCurrentTarget().is_none());

        // Step 1.
        self.dispatching.set(true);

        // Step 2.
        self.target.set(Some(target_override.unwrap_or(target)));

        if self.stop_propagation.get() {
            // If the event's stop propagation flag is set, we can skip everything because
            // it prevents the calls of the invoke algorithm in the spec.

            // Step 10-12.
            self.clear_dispatching_flags();

            // Step 14.
            return self.status();
        }

        // Step 3. The "invoke" algorithm is only used on `target` separately,
        // so we don't put it in the path.
        rooted_vec!(let mut event_path);

        // Step 4.
        if let Some(target_node) = target.downcast::<Node>() {
            for ancestor in target_node.ancestors() {
                event_path.push(JS::from_ref(ancestor.upcast::<EventTarget>()));
            }
            let top_most_ancestor_or_target =
                Root::from_ref(event_path.r().last().cloned().unwrap_or(target));
            if let Some(document) = Root::downcast::<Document>(top_most_ancestor_or_target) {
                if self.type_() != atom!("load") && document.browsing_context().is_some() {
                    event_path.push(JS::from_ref(document.window().upcast()));
                }
            }
        }

        // Steps 5-9. In a separate function to short-circuit various things easily.
        dispatch_to_listeners(self, target, event_path.r());

        // Default action.
        if let Some(target) = self.GetTarget() {
            if let Some(node) = target.downcast::<Node>() {
                let vtable = vtable_for(&node);
                vtable.handle_event(self);
            }
        }

        // Step 10-12.
        self.clear_dispatching_flags();

        // Step 14.
        self.status()
    }

    pub fn status(&self) -> EventStatus {
        match self.DefaultPrevented() {
            true => EventStatus::Canceled,
            false => EventStatus::NotCanceled
        }
    }

    #[inline]
    pub fn dispatching(&self) -> bool {
        self.dispatching.get()
    }

    #[inline]
    // https://dom.spec.whatwg.org/#concept-event-dispatch Steps 10-12.
    fn clear_dispatching_flags(&self) {
        assert!(self.dispatching.get());

        self.dispatching.set(false);
        self.stop_propagation.set(false);
        self.stop_immediate.set(false);
        self.phase.set(EventPhase::None);
        self.current_target.set(None);
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

    // https://html.spec.whatwg.org/multipage/#fire-a-simple-event
    pub fn fire(&self, target: &EventTarget) -> EventStatus {
        self.set_trusted(true);
        target.dispatch_event(self)
    }
}

impl EventMethods for Event {
    // https://dom.spec.whatwg.org/#dom-event-eventphase
    fn EventPhase(&self) -> u16 {
        self.phase.get() as u16
    }

    // https://dom.spec.whatwg.org/#dom-event-type
    fn Type(&self) -> DOMString {
        DOMString::from(&*self.type_()) // FIXME(ajeffrey): Directly convert from Atom to DOMString
    }

    // https://dom.spec.whatwg.org/#dom-event-target
    fn GetTarget(&self) -> Option<Root<EventTarget>> {
        self.target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-currenttarget
    fn GetCurrentTarget(&self) -> Option<Root<EventTarget>> {
        self.current_target.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-defaultprevented
    fn DefaultPrevented(&self) -> bool {
        self.canceled.get() == EventDefault::Prevented
    }

    // https://dom.spec.whatwg.org/#dom-event-preventdefault
    fn PreventDefault(&self) {
        if self.cancelable.get() {
            self.canceled.set(EventDefault::Prevented)
        }
    }

    // https://dom.spec.whatwg.org/#dom-event-stoppropagation
    fn StopPropagation(&self) {
        self.stop_propagation.set(true);
    }

    // https://dom.spec.whatwg.org/#dom-event-stopimmediatepropagation
    fn StopImmediatePropagation(&self) {
        self.stop_immediate.set(true);
        self.stop_propagation.set(true);
    }

    // https://dom.spec.whatwg.org/#dom-event-bubbles
    fn Bubbles(&self) -> bool {
        self.bubbles.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-cancelable
    fn Cancelable(&self) -> bool {
        self.cancelable.get()
    }

    // https://dom.spec.whatwg.org/#dom-event-timestamp
    fn TimeStamp(&self) -> u64 {
        self.timestamp
    }

    // https://dom.spec.whatwg.org/#dom-event-initevent
    fn InitEvent(&self,
                 type_: DOMString,
                 bubbles: bool,
                 cancelable: bool) {
         self.init_event(Atom::from(type_), bubbles, cancelable)
    }

    // https://dom.spec.whatwg.org/#dom-event-istrusted
    fn IsTrusted(&self) -> bool {
        self.trusted.get()
    }
}

#[derive(PartialEq, HeapSizeOf, Copy, Clone)]
pub enum EventBubbles {
    Bubbles,
    DoesNotBubble
}

impl From<bool> for EventBubbles {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => EventBubbles::Bubbles,
            false => EventBubbles::DoesNotBubble
        }
    }
}

impl From<EventBubbles> for bool {
    fn from(bubbles: EventBubbles) -> Self {
        match bubbles {
            EventBubbles::Bubbles => true,
            EventBubbles::DoesNotBubble => false
        }
    }
}

#[derive(PartialEq, HeapSizeOf, Copy, Clone)]
pub enum EventCancelable {
    Cancelable,
    NotCancelable
}

impl From<bool> for EventCancelable {
    fn from(boolean: bool) -> Self {
        match boolean {
            true => EventCancelable::Cancelable,
            false => EventCancelable::NotCancelable
        }
    }
}

impl From<EventCancelable> for bool {
    fn from(bubbles: EventCancelable) -> Self {
        match bubbles {
            EventCancelable::Cancelable => true,
            EventCancelable::NotCancelable => false
        }
    }
}

#[derive(JSTraceable, Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
#[derive(HeapSizeOf)]
pub enum EventPhase {
    None      = EventConstants::NONE,
    Capturing = EventConstants::CAPTURING_PHASE,
    AtTarget  = EventConstants::AT_TARGET,
    Bubbling  = EventConstants::BUBBLING_PHASE,
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
/// [msg]: https://doc.servo.org/script_traits/enum.ConstellationMsg.html#variant.KeyEvent
///
#[derive(JSTraceable, HeapSizeOf, Copy, Clone, PartialEq)]
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
    NotCanceled
}

// https://dom.spec.whatwg.org/#concept-event-fire
pub struct EventRunnable {
    pub target: Trusted<EventTarget>,
    pub name: Atom,
    pub bubbles: EventBubbles,
    pub cancelable: EventCancelable,
}

impl Runnable for EventRunnable {
    fn name(&self) -> &'static str { "EventRunnable" }

    fn handler(self: Box<EventRunnable>) {
        let target = self.target.root();
        let bubbles = self.bubbles;
        let cancelable = self.cancelable;
        target.fire_event_with_params(self.name, bubbles, cancelable);
    }
}

// https://html.spec.whatwg.org/multipage/#fire-a-simple-event
pub struct SimpleEventRunnable {
    pub target: Trusted<EventTarget>,
    pub name: Atom,
}

impl Runnable for SimpleEventRunnable {
    fn name(&self) -> &'static str { "SimpleEventRunnable" }

    fn handler(self: Box<SimpleEventRunnable>) {
        let target = self.target.root();
        target.fire_event(self.name);
    }
}

// See dispatch_event.
// https://dom.spec.whatwg.org/#concept-event-dispatch
fn dispatch_to_listeners(event: &Event, target: &EventTarget, event_path: &[&EventTarget]) {
    assert!(!event.stop_propagation.get());
    assert!(!event.stop_immediate.get());

    let window = match Root::downcast::<Window>(target.global()) {
        Some(window) => {
            if window.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                Some(window)
            } else {
                None
            }
        },
        _ => None,
    };

    // Step 5.
    event.phase.set(EventPhase::Capturing);

    // Step 6.
    for object in event_path.iter().rev() {
        invoke(window.r(), object, event, Some(ListenerPhase::Capturing));
        if event.stop_propagation.get() {
            return;
        }
    }
    assert!(!event.stop_propagation.get());
    assert!(!event.stop_immediate.get());

    // Step 7.
    event.phase.set(EventPhase::AtTarget);

    // Step 8.
    invoke(window.r(), target, event, None);
    if event.stop_propagation.get() {
        return;
    }
    assert!(!event.stop_propagation.get());
    assert!(!event.stop_immediate.get());

    if !event.bubbles.get() {
        return;
    }

    // Step 9.1.
    event.phase.set(EventPhase::Bubbling);

    // Step 9.2.
    for object in event_path {
        invoke(window.r(), object, event, Some(ListenerPhase::Bubbling));
        if event.stop_propagation.get() {
            return;
        }
    }
}

// https://dom.spec.whatwg.org/#concept-event-listener-invoke
fn invoke(window: Option<&Window>,
          object: &EventTarget,
          event: &Event,
          specific_listener_phase: Option<ListenerPhase>) {
    // Step 1.
    assert!(!event.stop_propagation.get());

    // Steps 2-3.
    let listeners = object.get_listeners_for(&event.type_(), specific_listener_phase);

    // Step 4.
    event.current_target.set(Some(object));

    // Step 5.
    inner_invoke(window, object, event, &listeners);

    // TODO: step 6.
}

// https://dom.spec.whatwg.org/#concept-event-listener-inner-invoke
fn inner_invoke(window: Option<&Window>,
                object: &EventTarget,
                event: &Event,
                listeners: &[CompiledEventListener])
                -> bool {
    // Step 1.
    let mut found = false;

    // Step 2.
    for listener in listeners {
        // Steps 2.1 and 2.3-2.4 are not done because `listeners` contain only the
        // relevant ones for this invoke call during the dispatch algorithm.

        // Step 2.2.
        found = true;

        // TODO: step 2.5.

        // Step 2.6.
        let marker = TimelineMarker::start("DOMEvent".to_owned());
        listener.call_or_handle_event(object, event, ExceptionHandling::Report);
        if let Some(window) = window {
            window.emit_timeline_marker(marker.end());
        }
        if event.stop_immediate.get() {
            return found;
        }

        // TODO: step 2.7.
    }

    // Step 3.
    found
}
