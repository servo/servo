/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast};
use dom::bindings::js::JS;
use dom::bindings::trace::RootedVec;
use dom::eventtarget::{EventTarget, ListenerPhase};
use dom::event::{Event, EventPhase};
use dom::node::{Node, NodeHelpers};
use dom::virtualmethods::vtable_for;

// See https://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event<'a, 'b>(target: &'a EventTarget,
                              pseudo_target: Option<&'b EventTarget>,
                              event: &Event) -> bool {
    assert!(!event.dispatching());
    assert!(event.initialized());

    event.set_target(match pseudo_target {
        Some(pseudo_target) => pseudo_target,
        None => target.clone(),
    });
    event.set_dispatching(true);

    let type_ = event.Type();

    //TODO: no chain if not participating in a tree
    let mut chain: RootedVec<JS<EventTarget>> = RootedVec::new();
    if let Some(target_node) = NodeCast::to_ref(target) {
        for ancestor in target_node.ancestors() {
            let ancestor_target = EventTargetCast::from_ref(ancestor.r());
            chain.push(JS::from_ref(ancestor_target))
        }
    }

    event.set_phase(EventPhase::Capturing);

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for cur_target in chain.r().iter().rev() {
        let stopped = match cur_target.get_listeners_for(&type_, ListenerPhase::Capturing) {
            Some(listeners) => {
                event.set_current_target(cur_target);
                for listener in &listeners {
                    // Explicitly drop any exception on the floor.
                    let _ = listener.HandleEvent_(*cur_target, event, Report);

                    if event.stop_immediate() {
                        break;
                    }
                }

                event.stop_propagation()
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.stop_propagation() {
        event.set_phase(EventPhase::AtTarget);
        event.set_current_target(target.clone());

        let opt_listeners = target.get_listeners(&type_);
        for listeners in opt_listeners {
            for listener in listeners {
                // Explicitly drop any exception on the floor.
                let _ = listener.HandleEvent_(target, event, Report);

                if event.stop_immediate() {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.bubbles() && !event.stop_propagation() {
        event.set_phase(EventPhase::Bubbling);

        for cur_target in chain.r() {
            let stopped = match cur_target.get_listeners_for(&type_, ListenerPhase::Bubbling) {
                Some(listeners) => {
                    event.set_current_target(cur_target);
                    for listener in &listeners {
                        // Explicitly drop any exception on the floor.
                        let _ = listener.HandleEvent_(*cur_target, event, Report);

                        if event.stop_immediate() {
                            break;
                        }
                    }

                    event.stop_propagation()
                }
                None => false
            };
            if stopped {
                break;
            }
        }
    }

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
