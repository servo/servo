/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::eReportExceptions;
use dom::bindings::jsmanaged::JSManaged;
use dom::eventtarget::{AbstractEventTarget, Capturing, Bubbling};
use dom::event::{Event, Phase_At_Target, Phase_None, Phase_Bubbling, Phase_Capturing};
use dom::node::AbstractNode;
use servo_util::tree::{TreeNodeRef};

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: AbstractEventTarget,
                      pseudo_target: Option<AbstractEventTarget>,
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
        for ancestor in AbstractNode::from_eventtarget(target).ancestors() {
            chain.push(AbstractEventTarget::from_node(ancestor));
        }
    }

    event.mut_value().phase = Phase_Capturing;

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for &cur_target in chain.rev_iter() {
        let stopped = match cur_target.eventtarget().get_listeners_for(type_, Capturing) {
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

        let opt_listeners = target.eventtarget().get_listeners(type_);
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
            let stopped = match cur_target.eventtarget().get_listeners_for(type_, Bubbling) {
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
