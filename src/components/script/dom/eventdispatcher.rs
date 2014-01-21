/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::eReportExceptions;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, NodeCast, NodeDerived};
use dom::bindings::jsmanaged::JSManaged;
use dom::eventtarget::{Capturing, Bubbling, EventTarget};
use dom::event::{Event, Phase_At_Target, Phase_None, Phase_Bubbling, Phase_Capturing};
use dom::node::Node;

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: JSManaged<EventTarget>,
                      pseudo_target: Option<JSManaged<EventTarget>>,
                      mut event: JSManaged<Event>) -> bool {
    assert!(!event.value().dispatching);

    {
        let event = event.mut_value();
        event.target = Some(pseudo_target.unwrap_or(target));
        event.dispatching = true;
    }

    let type_ = event.value().type_.clone();
    let mut chain = ~[];

    //TODO: no chain if not participating in a tree
    if target.is_node() {
        let target_node: JSManaged<Node> = NodeCast::to(target);
        for ancestor in target_node.value().ancestors() {
            let ancestor_target: JSManaged<EventTarget> = EventTargetCast::from(ancestor);
            chain.push(ancestor_target);
        }
    }

    event.mut_value().phase = Phase_Capturing;

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for &cur_target in chain.rev_iter() {
        let stopped = match cur_target.value().get_listeners_for(type_, Capturing) {
            Some(listeners) => {
                event.mut_value().current_target = Some(cur_target);
                for listener in listeners.iter() {
                    listener.HandleEvent__(event, eReportExceptions);

                    if event.value().stop_immediate {
                        break;
                    }
                }

                event.value().stop_propagation
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.value().stop_propagation {
        {
            let event = event.mut_value();
            event.phase = Phase_At_Target;
            event.current_target = Some(target);
        }

        let opt_listeners = target.value().get_listeners(type_);
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
                listener.HandleEvent__(event, eReportExceptions);
                if event.value().stop_immediate {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.value().bubbles && !event.value().stop_propagation {
        event.mut_value().phase = Phase_Bubbling;

        for &cur_target in chain.iter() {
            let stopped = match cur_target.value().get_listeners_for(type_, Bubbling) {
                Some(listeners) => {
                    event.mut_value().current_target = Some(cur_target);
                    for listener in listeners.iter() {
                        listener.HandleEvent__(event, eReportExceptions);

                        if event.value().stop_immediate {
                            break;
                        }
                    }

                    event.value().stop_propagation
                }
                None => false
            };
            if stopped {
                break;
            }
        }
    }

    let event = event.mut_value();
    event.dispatching = false;
    event.phase = Phase_None;
    event.current_target = None;

    !event.DefaultPrevented()
}
