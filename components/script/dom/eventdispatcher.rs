/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast};
use dom::bindings::global::{GlobalRoot, global_object_for_reflector};
use dom::bindings::js::{JS, Root, RootedReference};
use dom::bindings::trace::RootedVec;
use dom::event::{Event, EventPhase};
use dom::eventtarget::{EventTarget, ListenerPhase, EventListenerType};
use dom::node::Node;
use dom::virtualmethods::vtable_for;
use dom::window::Window;

use devtools_traits::{StartedTimelineMarker, TimelineMarker, TimelineMarkerType};

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

fn handle_event(window: Option<&Window>, listener: &EventListenerType,
                current_target: &EventTarget, event: &Event) {
    let _marker;
    if let Some(window) = window {
        _marker = AutoDOMEventMarker::new(window);
    }

    listener.call_or_handle_event(current_target, event, Report);
}

fn dispatch_to_listeners(event: &Event, target: &EventTarget, chain: &[&EventTarget]) {
    let window = match global_object_for_reflector(target) {
        GlobalRoot::Window(window) => {
            if window.need_emit_timeline_marker(TimelineMarkerType::DOMEvent) {
                Some(window)
            } else {
                None
            }
        },
        _ => None,
    };

    let type_ = event.Type();

    /* capturing */
    event.set_phase(EventPhase::Capturing);
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

    /* at target */
    event.set_phase(EventPhase::AtTarget);
    event.set_current_target(target);

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

    /* bubbling */
    if event.bubbles() {
        event.set_phase(EventPhase::Bubbling);

        for cur_target in chain {
            if let Some(listeners) = cur_target.get_listeners_for(&type_, ListenerPhase::Bubbling) {
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
}

// See https://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: &EventTarget, pseudo_target: Option<&EventTarget>,
                      event: &Event) -> bool {
    assert!(!event.dispatching());
    assert!(event.initialized());

    event.set_target(match pseudo_target {
        Some(pseudo_target) => pseudo_target,
        None => target.clone(),
    });
    event.set_dispatching(true);

    //TODO: no chain if not participating in a tree
    let mut chain: RootedVec<JS<EventTarget>> = RootedVec::new();
    if let Some(target_node) = NodeCast::to_ref(target) {
        for ancestor in target_node.ancestors() {
            let ancestor_target = EventTargetCast::from_ref(ancestor.r());
            chain.push(JS::from_ref(ancestor_target))
        }
    }

    dispatch_to_listeners(event, target, chain.r());

    /* default action */
    let target = event.GetTarget();
    match target {
        Some(ref target) => {
            let node: Option<&Node> = NodeCast::to_ref(target.r());
            match node {
                Some(node) => {
                    let vtable = vtable_for(&node);
                    vtable.handle_event(event);
                }
                None => {}
            }
        }
        None => {}
    }

    event.set_dispatching(false);
    event.set_phase(EventPhase::None);
    event.clear_current_target();

    !event.DefaultPrevented()
}
