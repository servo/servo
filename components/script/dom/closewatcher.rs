/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr::NonNull;

use dom_struct::dom_struct;
use js::gc::HandleObject;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::inheritance::Castable;
use script_bindings::root::{Dom, DomRoot};
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CloseWatcherBinding::{
    CloseWatcherMethods, CloseWatcherOptions,
};
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object_with_proto};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmldialogelement::HTMLDialogElement;
use crate::dom::window::Window;

/// <https://html.spec.whatwg.org/multipage/#close-watcher>
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct InternalCloseWatcher {
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-window>
    window: Dom<Window>,
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-is-running-cancel>
    is_running_cancel_action: Cell<bool>,

    active: Cell<bool>,
    owner: InternalCloseWatcherOwner,
}

impl InternalCloseWatcher {
    pub fn new(window: &Window, owner: InternalCloseWatcherOwner) -> Self {
        Self {
            window: Dom::from_ref(window),
            is_running_cancel_action: Cell::new(false),
            active: Cell::new(true),
            owner,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#establish-a-close-watcher>
    pub fn establish(window: &Window, owner: InternalCloseWatcherOwner) -> InternalCloseWatcher {
        // Step 1. Assert: window's associated Document is fully active.
        assert!(window.Document().is_fully_active());
        // Step 2. Let closeWatcher be a new close watcher...
        let close_watcher = Self::new(window, owner);
        // Step 3. Let manager be window's close watcher manager.
        let mut manager = window.close_watcher_manager_mut();
        let pointer = NonNull::from(&close_watcher);
        // Step 4. If manager's groups's size is less than manager's allowed number of groups, then append « closeWatcher » to manager's groups.
        if manager.number_of_groups() < manager.allowed_number_of_groups() {
            manager.append_group(vec![pointer])
        } else {
            // Step 5. Otherwise:
            // Step 5.1. Assert: manager's groups's size is at least 1 in this branch, since manager's allowed number of groups is always at least 1.
            assert!(manager.number_of_groups() >= 1);
            // Step 5.2. Append closeWatcher to manager's groups's last item.
            manager.append_close_watcher(pointer)
        }
        // Step 6. Set manager's next user interaction allows a new group to true.
        manager.set_next_user_interaction_allows_a_new_group(true);
        // Step 7. Return closeWatcher.
        close_watcher
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-request-close>
    pub fn request_to_close(&self, require_history_action_activation: bool, can_gc: CanGc) -> bool {
        // Step 1. If closeWatcher is not active, then return true.
        if !self.active.get() {
            return true;
        }
        // Step 2. If the result of running closeWatcher's get enabled state is false, then return true.
        if !self.get_enabled_state() {
            return true;
        }
        // Step 3. If closeWatcher's is running cancel action is true, then return true.
        if self.is_running_cancel_action.get() {
            return true;
        }
        // Step 4. Let window be closeWatcher's window.
        let window = &self.window;
        // Step 5. If window's associated Document is not fully active, then return true.
        if !window.Document().is_fully_active() {
            return true;
        }
        // Step 6. Let canPreventClose be true if requireHistoryActionActivation is false, or ...; otherwise false.
        let can_prevent_close = !require_history_action_activation || {
            let manager = window.close_watcher_manager();
            // if window's close watcher manager's groups's size is less than window's close watcher manager's allowed number of groups, and window has history-action activation
            manager.number_of_groups() < manager.allowed_number_of_groups() &&
                window.has_history_action_activation()
        };
        // Step 7. Set closeWatcher's is running cancel action to true.
        self.is_running_cancel_action.set(true);
        // Step 8. Let shouldContinue be the result of running closeWatcher's cancel action given canPreventClose.
        let should_continue = self.cancel_action(can_prevent_close, can_gc);
        // Step 9. Set closeWatcher's is running cancel action to false.
        self.is_running_cancel_action.set(false);
        // Step 10. If shouldContinue is false, then:
        if !should_continue {
            // Step 10.1. Assert: canPreventClose is true.
            assert!(can_prevent_close);
            // Step 10.2. Consume history-action user activation given window.
            window.consume_history_action_user_activation();
            // Step 10.3. Return false.
            return false;
        }
        // Step 11. Close closeWatcher.
        self.close(can_gc);
        // Step 12. Return true.
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-close>
    pub fn close(&self, can_gc: CanGc) {
        // Step 1. If closeWatcher is not active, then return.
        if !self.active.get() {
            return;
        }
        // Step 2. If the result of running closeWatcher's get enabled state is false, then return.
        if !self.get_enabled_state() {
            return;
        }
        // Step 3. If closeWatcher's window's associated Document is not fully active, then return.
        if !self.window.Document().is_fully_active() {
            return;
        }
        // Step 4. Destroy closeWatcher.
        self.destroy();
        // Step 5. Run closeWatcher's close action.
        self.close_action(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-destroy>
    pub fn destroy(&self) {
        // Step 1. Let manager be closeWatcher's window's close watcher manager.
        let mut manager = self.window.close_watcher_manager_mut();
        // Steps 2-3 handled inside remove_close_watcher.
        manager.remove_close_watcher(self);
        self.active.set(false)
    }

    /// <https://html.spec.whatwg.org/multipage/#create-close-watcher-cancelaction>
    pub fn cancel_action(&self, can_prevent_close: bool, can_gc: CanGc) -> bool {
        match &self.owner {
            InternalCloseWatcherOwner::CloseWatcher(close_watcher) => {
                // cancelAction given canPreventClose being to return the result of firing an event
                // named cancel at this, with the cancelable attribute initialized to
                // canPreventClose
                let event = Event::new(
                    &self.window.global(),
                    atom!("cancel"),
                    EventBubbles::DoesNotBubble,
                    if can_prevent_close {
                        EventCancelable::Cancelable
                    } else {
                        EventCancelable::NotCancelable
                    },
                    can_gc,
                );
                event.fire(close_watcher.upcast::<EventTarget>(), can_gc)
            },
            InternalCloseWatcherOwner::Dialog(dialog) => {
                // cancelAction given canPreventClose being to return the result of firing an event
                // named cancel at dialog, with the cancelable attribute initialized to
                // canPreventClose.
                let event = Event::new(
                    &self.window.global(),
                    atom!("cancel"),
                    EventBubbles::DoesNotBubble,
                    if can_prevent_close {
                        EventCancelable::Cancelable
                    } else {
                        EventCancelable::NotCancelable
                    },
                    can_gc,
                );
                event.fire(dialog.upcast::<EventTarget>(), can_gc)
                // - closeAction being to close the dialog given dialog, dialog's request close return value, and dialog's request close source element.
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-close-watcher-closeaction>
    pub fn close_action(&self, can_gc: CanGc) {
        match &self.owner {
            InternalCloseWatcherOwner::CloseWatcher(close_watcher) => {
                // closeAction being to fire an event named close at this.
                let event = Event::new(
                    &self.window.global(),
                    atom!("close"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    can_gc,
                );
                event.fire(close_watcher.upcast::<EventTarget>(), can_gc);
            },
            InternalCloseWatcherOwner::Dialog(dialog) => {
                // closeAction being to close the dialog given dialog, dialog's request close
                // return value, and dialog's request close source element.
                dialog.close_watcher_close(can_gc)
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-get-enabled-state>
    pub fn get_enabled_state(&self) -> bool {
        match &self.owner {
            InternalCloseWatcherOwner::Dialog(dialog) => dialog.close_watcher_enabled_state(),
            _ => true,
        }
    }
}

#[derive(JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum InternalCloseWatcherOwner {
    CloseWatcher(Dom<CloseWatcher>),
    Dialog(Dom<HTMLDialogElement>),
}

#[dom_struct]
pub(crate) struct CloseWatcher {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/interaction.html#internal-close-watcher>
    internal_close_watcher: DomRefCell<Option<InternalCloseWatcher>>,
}

impl CloseWatcher {
    pub fn new_inherited() -> CloseWatcher {
        let close_watcher = CloseWatcher {
            eventtarget: EventTarget::new_inherited(),
            internal_close_watcher: DomRefCell::new(None),
        };
        close_watcher
    }

    pub fn new_with_proto(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<CloseWatcher> {
        let close_watcher = reflect_dom_object_with_proto(
            Box::new(CloseWatcher::new_inherited()),
            window,
            proto,
            can_gc,
        );

        close_watcher
    }
}

#[allow(non_snake_case)]
impl CloseWatcherMethods<DomTypeHolder> for CloseWatcher {
    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher>
    fn Constructor(
        window: &Window,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        options: &CloseWatcherOptions,
    ) -> Fallible<DomRoot<CloseWatcher>> {
        // Step 1. If this's relevant global object's associated Document is not fully active, then throw an "InvalidStateError" DOMException.
        if !window.Document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }
        // Step 2. Let closeWatcher be the result of establishing a close watcher given this's relevant global object, with:
        // - cancelAction given canPreventClose being to return the result of firing an event named cancel at this, with the cancelable attribute initialized to canPreventClose.
        // - closeAction being to fire an event named close at this.
        // - getEnabledState being to return true.
        let close_watcher = CloseWatcher::new_with_proto(window, proto, can_gc);
        let internal_close_watcher = InternalCloseWatcher::establish(
            window,
            InternalCloseWatcherOwner::CloseWatcher(Dom::from_ref(&*close_watcher)),
        );
        // Step 3. If options["signal"] exists, then:
        if let Some(signal) = options.signal.as_ref() {
            // Step 3.1. If options["signal"] is aborted, then destroy closeWatcher.
            if signal.aborted() {
                internal_close_watcher.destroy()
            }
            // TODO Step 3.2. Add the following steps to options["signal"]:
            // TODO Step 3.2.1. Destroy closeWatcher.
        }
        // Step 4. Set this's internal close watcher to closeWatcher.
        close_watcher
            .internal_close_watcher
            .borrow_mut()
            .replace(internal_close_watcher);
        Ok(close_watcher)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-requestclose>
    fn RequestClose(&self, can_gc: CanGc) {
        // The requestClose() method steps are to request to close this's internal close watcher with false.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.request_to_close(false, can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-close>
    fn Close(&self, can_gc: CanGc) {
        // The requestClose() method steps are to close this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.close(can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-destroy>
    fn Destroy(&self) {
        // The requestClose() method steps are to destroy this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.destroy();
        }
    }

    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-oncancel>
    event_handler!(cancel, GetOncancel, SetOncancel);
    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-onclose>
    event_handler!(close, GetOnclose, SetOnclose);
}
