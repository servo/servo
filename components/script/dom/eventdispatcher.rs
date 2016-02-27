/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{StartedTimelineMarker, TimelineMarker, TimelineMarkerType};
use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::global::GlobalRoot;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::reflector::Reflectable;
use dom::bindings::trace::RootedVec;
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

fn dispatch_to_listeners(event: &Event, target: &EventTarget, chain: &[&EventTarget]) {
    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

    let window = match target.global() {
        GlobalRoot::Window(window) => {
            if window.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                Some(window)
            } else {
                None
            }
        },
        _ => None,
    };

    let type_ = event.type_();

    // Step 4.
    event.set_phase(EventPhase::Capturing);

    // Step 5.
    for cur_target in chain.iter().rev() {
        if let Some(listeners) = cur_target.get_listeners_for(&type_, ListenerPhase::Capturing) {
            event.set_current_target(cur_target);
            for listener in &listeners {
                handle_event(window.r(), listener, *cur_target, event);

                if event.stop_immediate() {
                    return;
                }
            }

            if event.stop_propagation() {
                return;
            }
        }
    }

    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

    // Step 6.
    event.set_phase(EventPhase::AtTarget);

    event.set_current_target(target);

    // Step 7.
    if let Some(listeners) = target.get_listeners(&type_) {
        for listener in listeners {
            handle_event(window.r(), &listener, target, event);

            if event.stop_immediate() {
                return;
            }
        }
        if event.stop_propagation() {
            return;
        }
    }

    assert!(!event.stop_propagation());
    assert!(!event.stop_immediate());

    // Step 8.
    if !event.bubbles() {
        return;
    }

    // Step 8.2
    event.set_phase(EventPhase::Bubbling);

    for cur_target in chain {
        if let Some(listeners) = cur_target.get_listeners_for(&type_, ListenerPhase::Bubbling) {
            // Step 8.3
            event.set_current_target(cur_target);
            for listener in &listeners {
                handle_event(window.r(), listener, *cur_target, event);

                if event.stop_immediate() {
                    return;
                }
            }

            if event.stop_propagation() {
                return;
            }
        }
    }
}

// See https://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: &EventTarget, pseudo_target: Option<&EventTarget>,
                      event: &Event) -> bool {
    assert!(!event.dispatching());
    assert!(event.initialized());
    assert_eq!(event.phase(), EventPhase::None);
    assert!(event.GetCurrentTarget().is_none());

    // Step 1.
    event.set_dispatching(true);

    // Step 2.
    event.set_target(match pseudo_target {
        Some(pseudo_target) => pseudo_target,
        None => target.clone(),
    });

    // Step 3.
    let mut chain: RootedVec<JS<EventTarget>> = RootedVec::new();
    if let Some(target_node) = target.downcast::<Node>() {
        for ancestor in target_node.ancestors() {
            chain.push(JS::from_ref(ancestor.upcast()));
        }
        let top_most_ancestor_or_target =
            Root::from_ref(chain.r().last().cloned().unwrap_or(target));
        if let Some(document) = Root::downcast::<Document>(top_most_ancestor_or_target) {
            if event.type_() != atom!("load") && document.browsing_context().is_some() {
                chain.push(JS::from_ref(document.window().upcast()));
            }
        }
    }

    // Steps 4 - 8
    dispatch_to_listeners(event, target, chain.r());

    /* default action */
    let target = event.GetTarget();
    match target {
        Some(ref target) => {
            if let Some(node) = target.downcast::<Node>() {
                let vtable = vtable_for(&node);
                vtable.handle_event(event);
            }
        }
        None => {}
    }

    // Step 9.
    event.set_dispatching(false);

    // Step 10.
    event.set_phase(EventPhase::None);

    // Step 11.
    event.clear_current_target();

    // Return false if event’s canceled flag is set, and true otherwise.
    !event.DefaultPrevented()
}
