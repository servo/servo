/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, NodeDerived};
use dom::bindings::js::{JS, JSRef, OptionalRootable, Root};
use dom::eventtarget::{EventTarget, ListenerPhase};
use dom::event::{Event, EventPhase};
use dom::node::{Node, NodeHelpers};
use dom::virtualmethods::vtable_for;

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event<'a, 'b>(target: JSRef<'a, EventTarget>,
                              pseudo_target: Option<JSRef<'b, EventTarget>>,
                              event: JSRef<Event>) -> bool {
    assert!(!event.dispatching());
    assert!(event.initialized());

    event.set_target(match pseudo_target {
        Some(pseudo_target) => pseudo_target,
        None => target.clone(),
    });
    event.set_dispatching(true);

    let type_ = event.Type();

    //TODO: no chain if not participating in a tree
    let mut chain: Vec<Root<EventTarget>> = if target.is_node() {
        let target_node: JSRef<Node> = NodeCast::to_ref(target).unwrap();
        target_node.ancestors().map(|ancestor| {
            let ancestor_target: JSRef<EventTarget> = EventTargetCast::from_ref(ancestor);
            JS::from_rooted(ancestor_target).root()
        }).collect()
    } else {
        vec!()
    };

    event.set_phase(EventPhase::Capturing);

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for cur_target in chain.as_slice().iter().rev() {
        let stopped = match cur_target.r().get_listeners_for(type_.as_slice(), ListenerPhase::Capturing) {
            Some(listeners) => {
                event.set_current_target(cur_target.r());
                for listener in listeners.iter() {
                    // Explicitly drop any exception on the floor.
                    let _ = listener.HandleEvent_(cur_target.r(), event, Report);

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

        let opt_listeners = target.get_listeners(type_.as_slice());
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
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

        for cur_target in chain.iter() {
            let stopped = match cur_target.r().get_listeners_for(type_.as_slice(), ListenerPhase::Bubbling) {
                Some(listeners) => {
                    event.set_current_target(cur_target.r());
                    for listener in listeners.iter() {
                        // Explicitly drop any exception on the floor.
                        let _ = listener.HandleEvent_(cur_target.r(), event, Report);

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
    let target = event.GetTarget().root();
    match target {
        Some(target) => {
            let node: Option<JSRef<Node>> = NodeCast::to_ref(target.r());
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

    // Root ordering restrictions mean we need to unroot the chain entries
    // in the same order they were rooted.
    while chain.len() > 0 {
        let _ = chain.pop();
    }

    event.set_dispatching(false);
    event.set_phase(EventPhase::None);
    event.clear_current_target();

    !event.DefaultPrevented()
}
