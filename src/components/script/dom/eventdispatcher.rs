/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ReportExceptions;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, NodeDerived};
use dom::bindings::js::JS;
use dom::eventtarget::{Capturing, Bubbling, EventTarget};
use dom::event::{Event, PhaseAtTarget, PhaseNone, PhaseBubbling, PhaseCapturing};
use dom::node::{Node, NodeHelpers};

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: &JS<EventTarget>,
                      pseudo_target: Option<JS<EventTarget>>,
                      event: &mut JS<Event>) -> bool {
    assert!(!event.get().dispatching);

    {
        let event = event.get_mut();
        event.target = pseudo_target.or_else(|| {
            Some(target.clone())
        });
        event.dispatching = true;
    }

    let type_ = event.get().type_.clone();

    //TODO: no chain if not participating in a tree
    let chain: Vec<JS<EventTarget>> = if target.get().is_node() {
        let target_node: JS<Node> = NodeCast::to(target).unwrap();
        target_node.ancestors().map(|ancestor| {
            let ancestor_target: JS<EventTarget> = EventTargetCast::from(&ancestor);
            ancestor_target
        }).collect()
    } else {
        vec!()
    };

    event.get_mut().phase = PhaseCapturing;

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for cur_target in chain.as_slice().rev_iter() {
        let stopped = match cur_target.get().get_listeners_for(type_, Capturing) {
            Some(listeners) => {
                event.get_mut().current_target = Some(cur_target.clone());
                for listener in listeners.iter() {
                    //FIXME: this should have proper error handling, or explicitly
                    //       drop the exception on the floor
                    assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());

                    if event.get().stop_immediate {
                        break;
                    }
                }

                event.get().stop_propagation
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.get().stop_propagation {
        {
            let event = event.get_mut();
            event.phase = PhaseAtTarget;
            event.current_target = Some(target.clone());
        }

        let opt_listeners = target.get().get_listeners(type_);
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
                //FIXME: this should have proper error handling, or explicitly drop the
                //       exception on the floor.
                assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());
                if event.get().stop_immediate {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.get().bubbles && !event.get().stop_propagation {
        event.get_mut().phase = PhaseBubbling;

        for cur_target in chain.iter() {
            let stopped = match cur_target.get().get_listeners_for(type_, Bubbling) {
                Some(listeners) => {
                    event.get_mut().current_target = Some(cur_target.clone());
                    for listener in listeners.iter() {
                        //FIXME: this should have proper error handling or explicitly
                        //       drop exceptions on the floor.
                        assert!(listener.HandleEvent__(event, ReportExceptions).is_ok());

                        if event.get().stop_immediate {
                            break;
                        }
                    }

                    event.get().stop_propagation
                }
                None => false
            };
            if stopped {
                break;
            }
        }
    }

    let event = event.get_mut();
    event.dispatching = false;
    event.phase = PhaseNone;
    event.current_target = None;

    !event.DefaultPrevented()
}
