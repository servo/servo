/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ReportExceptions;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, NodeDerived};
use dom::bindings::js::{JS, JSRef, OptionalSettable, OptionalRootable, Root};
use dom::eventtarget::{Capturing, Bubbling, EventTarget};
use dom::event::{Event, PhaseAtTarget, PhaseNone, PhaseBubbling, PhaseCapturing};
use dom::node::{Node, NodeHelpers};
use dom::virtualmethods::vtable_for;

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event<'a, 'b>(target: JSRef<'a, EventTarget>,
                              pseudo_target: Option<JSRef<'b, EventTarget>>,
                              event: JSRef<Event>) -> bool {
    assert!(!event.deref().dispatching.deref().get());

    event.target.assign(Some(match pseudo_target {
        Some(pseudo_target) => pseudo_target,
        None => target.clone(),
    }));
    event.dispatching.deref().set(true);

    let type_ = event.Type();

    //TODO: no chain if not participating in a tree
    let mut chain: Vec<Root<EventTarget>> = if target.deref().is_node() {
        let target_node: JSRef<Node> = NodeCast::to_ref(target).unwrap();
        target_node.ancestors().map(|ancestor| {
            let ancestor_target: JSRef<EventTarget> = EventTargetCast::from_ref(ancestor);
            JS::from_rooted(ancestor_target).root()
        }).collect()
    } else {
        vec!()
    };

    event.deref().phase.deref().set(PhaseCapturing);

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for cur_target in chain.as_slice().iter().rev() {
        let stopped = match cur_target.get_listeners_for(type_.as_slice(), Capturing) {
            Some(listeners) => {
                event.current_target.assign(Some(cur_target.deref().clone()));
                for listener in listeners.iter() {
                    // Explicitly drop any exception on the floor.
                    let _ = listener.HandleEvent_(**cur_target, event, ReportExceptions);

                    if event.deref().stop_immediate.deref().get() {
                        break;
                    }
                }

                event.deref().stop_propagation.deref().get()
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.deref().stop_propagation.deref().get() {
        event.phase.deref().set(PhaseAtTarget);
        event.current_target.assign(Some(target.clone()));

        let opt_listeners = target.deref().get_listeners(type_.as_slice());
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
                // Explicitly drop any exception on the floor.
                let _ = listener.HandleEvent_(target, event, ReportExceptions);

                if event.deref().stop_immediate.deref().get() {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.deref().bubbles.deref().get() && !event.deref().stop_propagation.deref().get() {
        event.deref().phase.deref().set(PhaseBubbling);

        for cur_target in chain.iter() {
            let stopped = match cur_target.deref().get_listeners_for(type_.as_slice(), Bubbling) {
                Some(listeners) => {
                    event.deref().current_target.assign(Some(cur_target.deref().clone()));
                    for listener in listeners.iter() {
                        // Explicitly drop any exception on the floor.
                        let _ = listener.HandleEvent_(**cur_target, event, ReportExceptions);

                        if event.deref().stop_immediate.deref().get() {
                            break;
                        }
                    }

                    event.deref().stop_propagation.deref().get()
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
            let node: Option<JSRef<Node>> = NodeCast::to_ref(*target);
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

    event.dispatching.deref().set(false);
    event.phase.deref().set(PhaseNone);
    event.current_target.set(None);

    !event.DefaultPrevented()
}
