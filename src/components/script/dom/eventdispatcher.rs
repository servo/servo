/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::eReportExceptions;
use dom::eventtarget::{AbstractEventTarget, Capturing, Bubbling};
use dom::event::{AbstractEvent, Phase_At_Target, Phase_None, Phase_Bubbling, Phase_Capturing};
use dom::node::AbstractNode;

// See http://dom.spec.whatwg.org/#concept-event-dispatch for the full dispatch algorithm
pub fn dispatch_event(target: AbstractEventTarget,
                      pseudo_target: Option<AbstractEventTarget>,
                      event: AbstractEvent) -> bool {
    assert!(!event.event().dispatching);

    {
        let event = event.mut_event();
        event.target = Some(pseudo_target.unwrap_or(target));
        event.dispatching = true;
    }

    let type_ = event.event().type_.clone();
    let mut chain = ~[];

    //TODO: no chain if not participating in a tree
    if target.is_node() {
        for ancestor in AbstractNode::from_eventtarget(target).ancestors() {
            chain.push(AbstractEventTarget::from_node(ancestor));
        }
    }

    event.mut_event().phase = Phase_Capturing;

    //FIXME: The "callback this value" should be currentTarget

    /* capturing */
    for &cur_target in chain.rev_iter() {
        let stopped = match cur_target.eventtarget().get_listeners_for(type_, Capturing) {
            Some(listeners) => {
                event.mut_event().current_target = Some(cur_target);
                for listener in listeners.iter() {
                    listener.HandleEvent__(event, eReportExceptions);

                    if event.event().stop_immediate {
                        break;
                    }
                }

                event.propagation_stopped()
            }
            None => false
        };

        if stopped {
            break;
        }
    }

    /* at target */
    if !event.propagation_stopped() {
        {
            let event = event.mut_event();
            event.phase = Phase_At_Target;
            event.current_target = Some(target);
        }

        let opt_listeners = target.eventtarget().get_listeners(type_);
        for listeners in opt_listeners.iter() {
            for listener in listeners.iter() {
                listener.HandleEvent__(event, eReportExceptions);
                if event.event().stop_immediate {
                    break;
                }
            }
        }
    }

    /* bubbling */
    if event.bubbles() && !event.propagation_stopped() {
        event.mut_event().phase = Phase_Bubbling;

        for &cur_target in chain.iter() {
            let stopped = match cur_target.eventtarget().get_listeners_for(type_, Bubbling) {
                Some(listeners) => {
                    event.mut_event().current_target = Some(cur_target);
                    for listener in listeners.iter() {
                        listener.HandleEvent__(event, eReportExceptions);

                        if event.event().stop_immediate {
                            break;
                        }
                    }

                    event.propagation_stopped()
                }
                None => false
            };
            if stopped {
                break;
            }
        }
    }

    let event = event.mut_event();
    event.dispatching = false;
    event.phase = Phase_None;
    event.current_target = None;

    !event.DefaultPrevented()
}
