/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ReportExceptions;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, NodeDerived};
use dom::bindings::js::{JSRef, OptionalSettable, Root};
use dom::eventtarget::{Capturing, Bubbling, EventTarget};
use dom::event::{Event, PhaseAtTarget, PhaseNone, PhaseBubbling, PhaseCapturing, EventMethods};
use dom::node::{Node, NodeHelpers};

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event<'a, 'b>(target: &JSRef<'a, EventTarget>,
                              pseudo_target: Option<JSRef<'b, EventTarget>>,
                              event: &mut JSRef<Event>) -> bool {
    assert!(!event.deref().dispatching);

    {
        let event = event.deref_mut();
        event.target.assign(Some(match pseudo_target {
            Some(pseudo_target) => pseudo_target,
            None => target.clone(),
        }));
        event.dispatching = true;
    }

    let type_ = event.deref().type_.clone();

    //TODO: no chain if not participating in a tree
    let mut chain: Vec<Root<EventTarget>> = if target.deref().is_node() {
        let target_node: &JSRef<Node> = NodeCast::to_ref(target).unwrap();
        target_node.ancestors().map(|ancestor| {
            let ancestor_target: &JSRef<EventTarget> = EventTargetCast::from_ref(&ancestor);
            ancestor_target.unrooted().root()
        }).collect()
    } else {
        vec!()
    };

    event.deref_mut().phase = PhaseCapturing;

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for cur_target in chain.as_slice().rev_iter() {
        let stopped = match cur_target.get_listeners_for(type_, Capturing) {
            Some(listeners) => {
                event.current_target.assign(Some(cur_target.deref().clone()));
                for listener in listeners.iter() {
                    //FIXME: this should have proper error handling, or explicitly
                    //       drop the exception on the floor
                    assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());

                    if event.deref().stop_immediate {
                        break;
                    }
                }

                event.deref().stop_propagation
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.deref().stop_propagation {
        {
            let event = event.deref_mut();
            event.phase = PhaseAtTarget;
            event.current_target.assign(Some(target.clone()));
        }

        let opt_listeners = target.deref().get_listeners(type_);
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
                //FIXME: this should have proper error handling, or explicitly drop the
                //       exception on the floor.
                assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());
                if event.deref().stop_immediate {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.deref().bubbles && !event.deref().stop_propagation {
        event.deref_mut().phase = PhaseBubbling;

        for cur_target in chain.iter() {
            let stopped = match cur_target.deref().get_listeners_for(type_, Bubbling) {
                Some(listeners) => {
                    event.deref_mut().current_target.assign(Some(cur_target.deref().clone()));
                    for listener in listeners.iter() {
                        //FIXME: this should have proper error handling or explicitly
                        //       drop exceptions on the floor.
                        assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());

                        if event.deref().stop_immediate {
                            break;
                        }
                    }

                    event.deref().stop_propagation
                }
                None => false
            };
            if stopped {
                break;
            }
        }
    }

    // Root ordering restrictions mean we need to unroot the chain entries
    // in the same order they were rooted.
    while chain.len() > 0 {
        let _ = chain.pop();
    }

    event.dispatching = false;
    event.phase = PhaseNone;
    event.current_target = None;

    !event.DefaultPrevented()
}
