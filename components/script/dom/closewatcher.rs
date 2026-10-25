/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::ptr;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::gc::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::error::{Error, Fallible};
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_proto;
use script_bindings::root::{Dom, DomRoot};

use crate::dom::abortsignal::AbortAlgorithm;
use crate::dom::bindings::codegen::Bindings::CloseWatcherBinding::{
    CloseWatcherMethods, CloseWatcherOptions,
};
use crate::dom::bindings::codegen::DomTypeHolder::DomTypeHolder;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmldialogelement::HTMLDialogElement;
use crate::dom::window::Window;

/// <https://html.spec.whatwg.org/multipage/#close-watcher>
#[derive(JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_in_rc)]
pub(crate) struct InternalCloseWatcher {
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-window>
    window: Dom<Window>,
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-is-running-cancel>
    is_running_cancel_action: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-cancel-action>
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-close-action>
    /// <https://html.spec.whatwg.org/multipage/#close-watcher-get-enabled-state>
    owner: InternalCloseWatcherOwner,
}

impl InternalCloseWatcher {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(window: &Window, owner: InternalCloseWatcherOwner) -> Self {
        Self {
            window: Dom::from_ref(window),
            is_running_cancel_action: Cell::new(false),
            owner,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-active>
    pub(crate) fn is_active(&self) -> bool {
        // > A close watcher closeWatcher is active if closeWatcher's window's close watcher manager
        // > contains any list which contains closeWatcher.
        self.window
            .close_watcher_manager()
            .groups()
            .iter()
            .flatten()
            .any(|watcher| ptr::eq(Rc::as_ptr(watcher), self))
    }

    /// <https://html.spec.whatwg.org/multipage/#establish-a-close-watcher>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn establish(
        window: &Window,
        owner: InternalCloseWatcherOwner,
    ) -> Rc<InternalCloseWatcher> {
        // Step 1. Assert: window's associated Document is fully active.
        assert!(window.Document().is_fully_active());

        // Step 2. Let closeWatcher be a new close watcher, with
        // - window: window
        // - cancel action: cancelAction
        // - close action: closeAction
        // - is running cancel action: false
        // - get enabled state: getEnabledState
        let close_watcher = Rc::new(Self::new(window, owner));

        // Step 3. Let manager be window's close watcher manager.
        let mut manager = window.close_watcher_manager_mut();

        // Step 4. If manager's groups's size is less than manager's allowed number of groups, then append « closeWatcher » to manager's groups.
        if manager.groups().len() < manager.allowed_number_of_groups() {
            manager.groups_mut().push(vec![close_watcher.clone()])
        }
        // Step 5. Otherwise:
        else {
            // Step 5.1. Assert: manager's groups's size is at least 1 in this branch, since manager's allowed number of groups is always at least 1.
            assert!(!manager.groups().is_empty());

            // Step 5.2. Append closeWatcher to manager's groups's last item.
            manager
                .groups_mut()
                .last_mut()
                .expect("There is at least one group at this point")
                .push(close_watcher.clone());
        }

        // Step 6. Set manager's next user interaction allows a new group to true.
        manager.set_next_user_interaction_allows_a_new_group(true);

        // Step 7. Return closeWatcher.
        close_watcher
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-request-close>
    pub(crate) fn request_to_close(
        &self,
        cx: &mut JSContext,
        require_history_action_activation: bool,
    ) -> bool {
        // Step 1. If closeWatcher is not active, then return true.
        if !self.is_active() {
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

        // Step 6. Let canPreventClose be true if requireHistoryActionActivation is false, or if
        // window's close watcher manager's groups's size is less than window's close watcher manager's
        // allowed number of groups, and window has history-action activation; otherwise false.
        let can_prevent_close = !require_history_action_activation || {
            let manager = window.close_watcher_manager();
            manager.groups().len() < manager.allowed_number_of_groups() &&
                window.has_history_action_activation()
        };

        // Step 7. Set closeWatcher's is running cancel action to true.
        self.is_running_cancel_action.set(true);

        // Step 8. Let shouldContinue be the result of running closeWatcher's cancel action given canPreventClose.
        let should_continue = self.cancel_action(cx, can_prevent_close);

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
        self.close(cx);

        // Step 12. Return true.
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-close>
    pub(crate) fn close(&self, cx: &mut JSContext) {
        // Step 1. If closeWatcher is not active, then return.
        if !self.is_active() {
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
        self.close_action(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-destroy>
    pub(crate) fn destroy(&self) {
        // Step 1. Let manager be closeWatcher's window's close watcher manager.
        let mut manager = self.window.close_watcher_manager_mut();

        // Step 2. For each group of manager's groups: remove closeWatcher from group.
        for group in manager.groups_mut() {
            group.retain(|watcher| !ptr::eq(Rc::as_ptr(watcher), self))
        }

        // Step 3. Remove any item from manager's groups that is empty.
        manager.groups_mut().retain(|group| !group.is_empty())
    }

    /// <https://html.spec.whatwg.org/multipage/#create-close-watcher-cancelaction>
    pub(crate) fn cancel_action(&self, cx: &mut JSContext, can_prevent_close: bool) -> bool {
        match &self.owner {
            InternalCloseWatcherOwner::CloseWatcher(close_watcher) => {
                // cancelAction given canPreventClose being to return the result of firing an event
                // named cancel at this, with the cancelable attribute initialized to
                // canPreventClose
                let event = Event::new(
                    cx,
                    &self.window.global(),
                    atom!("cancel"),
                    EventBubbles::DoesNotBubble,
                    if can_prevent_close {
                        EventCancelable::Cancelable
                    } else {
                        EventCancelable::NotCancelable
                    },
                );
                event.fire(cx, close_watcher.upcast::<EventTarget>())
            },
            InternalCloseWatcherOwner::Dialog(dialog) => {
                // cancelAction given canPreventClose being to return the result of firing an event
                // named cancel at dialog, with the cancelable attribute initialized to
                // canPreventClose.
                let event = Event::new(
                    cx,
                    &self.window.global(),
                    atom!("cancel"),
                    EventBubbles::DoesNotBubble,
                    if can_prevent_close {
                        EventCancelable::Cancelable
                    } else {
                        EventCancelable::NotCancelable
                    },
                );
                event.fire(cx, dialog.upcast::<EventTarget>())
                // - closeAction being to close the dialog given dialog, dialog's request close return value, and dialog's request close source element.
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#create-close-watcher-closeaction>
    pub(crate) fn close_action(&self, cx: &mut JSContext) {
        match &self.owner {
            InternalCloseWatcherOwner::CloseWatcher(close_watcher) => {
                // closeAction being to fire an event named close at this.
                let event = Event::new(
                    cx,
                    &self.window.global(),
                    atom!("close"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                );
                event.fire(cx, close_watcher.upcast::<EventTarget>());
            },
            InternalCloseWatcherOwner::Dialog(dialog) => {
                // closeAction being to close the dialog given dialog, dialog's request close
                // return value, and dialog's request close source element.
                dialog.close_watcher_close(cx)
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#close-watcher-get-enabled-state>
    pub(crate) fn get_enabled_state(&self) -> bool {
        match &self.owner {
            InternalCloseWatcherOwner::Dialog(dialog) => dialog.close_watcher_enabled_state(),
            _ => true,
        }
    }
}

#[derive(JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::allow_unrooted_in_rc)]
pub(crate) enum InternalCloseWatcherOwner {
    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher>
    CloseWatcher(Dom<CloseWatcher>),
    /// <https://html.spec.whatwg.org/multipage/#set-the-dialog-close-watcher>
    Dialog(Dom<HTMLDialogElement>),
}

#[dom_struct]
pub(crate) struct CloseWatcher {
    eventtarget: EventTarget,
    /// <https://html.spec.whatwg.org/multipage/#internal-close-watcher>
    #[conditional_malloc_size_of]
    internal_close_watcher: DomRefCell<Option<Rc<InternalCloseWatcher>>>,
}

impl CloseWatcher {
    pub(crate) fn new_inherited() -> CloseWatcher {
        Self {
            eventtarget: EventTarget::new_inherited(),
            internal_close_watcher: DomRefCell::new(None),
        }
    }

    pub(crate) fn new_with_proto(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
    ) -> DomRoot<CloseWatcher> {
        reflect_dom_object_with_proto(cx, Box::new(CloseWatcher::new_inherited()), window, proto)
    }
}

#[allow(non_snake_case)]
impl CloseWatcherMethods<DomTypeHolder> for CloseWatcher {
    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher>
    fn Constructor(
        cx: &mut JSContext,
        window: &Window,
        proto: Option<HandleObject>,
        options: &CloseWatcherOptions,
    ) -> Fallible<DomRoot<CloseWatcher>> {
        // Step 1. If this's relevant global object's associated Document is not fully active, then throw an "InvalidStateError" DOMException.
        if !window.Document().is_fully_active() {
            return Err(Error::InvalidState(Some("Failed to construct 'CloseWatcher': CloseWatchers cannot be created in detached Windows.".into())));
        }
        // Step 2. Let closeWatcher be the result of establishing a close watcher given this's relevant global object, with:
        // - cancelAction given canPreventClose being to return the result of firing an event named cancel at this, with the cancelable attribute initialized to canPreventClose.
        // - closeAction being to fire an event named close at this.
        // - getEnabledState being to return true.
        let close_watcher = CloseWatcher::new_with_proto(cx, window, proto);
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
            // Step 3.2. Add the following steps to options["signal"]:
            // Step 3.2.1. Destroy closeWatcher.
            signal.add(&AbortAlgorithm::CloseWatcher(
                internal_close_watcher.clone(),
            ));
        }
        // Step 4. Set this's internal close watcher to closeWatcher.
        close_watcher
            .internal_close_watcher
            .borrow_mut()
            .replace(internal_close_watcher);
        Ok(close_watcher)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-requestclose>
    fn RequestClose(&self, cx: &mut JSContext) {
        // > The requestClose() method steps are to request to close this's internal close watcher with false.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.request_to_close(cx, false);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-close>
    fn Close(&self, cx: &mut JSContext) {
        // > The requestClose() method steps are to close this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.close(cx);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-closewatcher-destroy>
    fn Destroy(&self) {
        // > The requestClose() method steps are to destroy this's internal close watcher.
        if let Some(internal) = &*self.internal_close_watcher.borrow() {
            internal.destroy();
        }
    }

    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-oncancel>
    event_handler!(cancel, GetOncancel, SetOncancel);
    // <https://html.spec.whatwg.org/multipage/#handler-closewatcher-onclose>
    event_handler!(close, GetOnclose, SetOnclose);
}
