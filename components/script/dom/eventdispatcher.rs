/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{StartedTimelineMarker, TimelineMarker, TimelineMarkerType};
use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::reflector::DomObject;
use dom::document::Document;
use dom::event::{Event, EventPhase};
use dom::eventtarget::{CompiledEventListener, EventTarget, ListenerPhase};
use dom::node::Node;
use dom::virtualmethods::vtable_for;
use dom::window::Window;

struct AutoDOMEventMarker {
    window: Root<Window>,
    marker: Option<StartedTimelineMarker>,
}

impl AutoDOMEventMarker {
    fn new(window: &Window) -> AutoDOMEventMarker {
        AutoDOMEventMarker {
            window: Root::from_ref(window),
            marker: Some(TimelineMarker::start("DOMEvent".to_owned())),
        }
    }
}

impl Drop for AutoDOMEventMarker {
    fn drop(&mut self) {
        self.window.emit_timeline_marker(self.marker.take().unwrap().end());
    }
}

fn handle_event(window: Option<&Window>, listener: &CompiledEventListener,
                current_target: &EventTarget, event: &Event) {
    let _marker;
    if let Some(window) = window {
        _marker = AutoDOMEventMarker::new(window);
    }

    listener.call_or_handle_event(current_target, event, Report);
}

// See dispatch_event.
// https://dom.spec.whatwg.org/#concept-event-dispatch
fn dispatch_to_listeners(event: &Event, target: &EventTarget, event_path: &[&EventTarget]) {
    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

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
    event.set_phase(EventPhase::Capturing);

    // Step 6.
    for object in event_path.iter().rev() {
        invoke(window.r(), object, event, Some(ListenerPhase::Capturing));
        if event.stop_propagation() {
            return;
        }
    }
    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

    // Step 7.
    event.set_phase(EventPhase::AtTarget);

    // Step 8.
    invoke(window.r(), target, event, None);
    if event.stop_propagation() {
        return;
    }
    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

    if !event.bubbles() {
        return;
    }

    // Step 9.1.
    event.set_phase(EventPhase::Bubbling);

    // Step 9.2.
    for object in event_path {
        invoke(window.r(), object, event, Some(ListenerPhase::Bubbling));
        if event.stop_propagation() {
            return;
        }
    }
}

#[derive(PartialEq)]
pub enum EventStatus {
    Canceled,
    NotCanceled
}

// https://dom.spec.whatwg.org/#concept-event-dispatch
pub fn dispatch_event(target: &EventTarget,
                      target_override: Option<&EventTarget>,
                      event: &Event) -> EventStatus {
    assert!(!event.dispatching());
    assert!(event.initialized());
    assert_eq!(event.phase(), EventPhase::None);
    assert!(event.GetCurrentTarget().is_none());

    // Step 1.
    event.mark_as_dispatching();

    // Step 2.
    event.set_target(target_override.unwrap_or(target));

    if event.stop_propagation() {
        // If the event's stop propagation flag is set, we can skip everything because
        // it prevents the calls of the invoke algorithm in the spec.

        // Step 10-12.
        event.clear_dispatching_flags();

        // Step 14.
        return event.status();
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
            if event.type_() != atom!("load") && document.browsing_context().is_some() {
                event_path.push(JS::from_ref(document.window().upcast()));
            }
        }
    }

    // Steps 5-9. In a separate function to short-circuit various things easily.
    dispatch_to_listeners(event, target, event_path.r());

    // Default action.
    if let Some(target) = event.GetTarget() {
        if let Some(node) = target.downcast::<Node>() {
            let vtable = vtable_for(&node);
            vtable.handle_event(event);
        }
    }

    // Step 10-12.
    event.clear_dispatching_flags();

    // Step 14.
    event.status()
}

// https://dom.spec.whatwg.org/#concept-event-listener-invoke
fn invoke(window: Option<&Window>,
          object: &EventTarget,
          event: &Event,
          specific_listener_phase: Option<ListenerPhase>) {
    // Step 1.
    assert!(!event.stop_propagation());

    // Steps 2-3.
    let listeners = object.get_listeners_for(&event.type_(), specific_listener_phase);

    // Step 4.
    event.set_current_target(object);

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
        handle_event(window, listener, object, event);
        if event.stop_immediate() {
            return found;
        }

        // TODO: step 2.7.
    }

    // Step 3.
    found
}
