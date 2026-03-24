/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::borrow::Borrow;
use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::error::{Error, ErrorResult};
use script_bindings::root::Dom;
use stylo_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::closewatcher::{InternalCloseWatcher, InternalCloseWatcherOwner};
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::htmlbuttonelement::{CommandState, HTMLButtonElement};
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::toggleevent::ToggleEvent;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DomRefCell<DOMString>,
    /// <https://html.spec.whatwg.org/multipage/#dialog-close-watcher>
    close_watcher: DomRefCell<Option<InternalCloseWatcher>>,
    /// <https://html.spec.whatwg.org/multipage/#request-close-return-value>
    request_close_return_value: DomRefCell<Option<DOMString>>,
    /// <https://html.spec.whatwg.org/multipage/#request-close-source-element>
    request_close_source_element: DomRefCell<Option<DomRoot<Element>>>,
    /// <https://html.spec.whatwg.org/multipage/#enable-close-watcher-for-requestclose()>
    enable_close_watcher_for_request_close: Cell<bool>,

    is_running_request_close: Cell<bool>,
}

impl HTMLDialogElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDialogElement {
        HTMLDialogElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            return_value: DomRefCell::new(DOMString::new()),
            close_watcher: DomRefCell::new(None),
            request_close_return_value: Default::default(),
            request_close_source_element: Default::default(),
            enable_close_watcher_for_request_close: Cell::new(false),
            is_running_request_close: Cell::new(false),
        }
    }

    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLDialogElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDialogElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#show-a-modal-dialog>
    pub fn show_a_modal(&self, source: Option<DomRoot<Element>>, can_gc: CanGc) -> ErrorResult {
        let subject = self.upcast::<Element>();
        // Step 1. If subject has an open attribute and is modal of subject is true, then return.
        if subject.has_attribute(&local_name!("open")) &&
            subject.state().contains(ElementState::MODAL)
        {
            return Ok(());
        }

        // Step 2. If subject has an open attribute, then throw an "InvalidStateError" DOMException.
        if subject.has_attribute(&local_name!("open")) {
            return Err(Error::InvalidState(Some(
                "Cannot call showModal() on an already open dialog.".into(),
            )));
        }

        // Step 3. If subject's node document is not fully active, then throw an "InvalidStateError" DOMException.
        if !subject.owner_document().is_fully_active() {
            return Err(Error::InvalidState(Some(
                "Cannot call showModal() on a dialog whose document is not fully active.".into(),
            )));
        }

        // Step 4. If subject is not connected, then throw an "InvalidStateError" DOMException.
        if !subject.is_connected() {
            return Err(Error::InvalidState(Some(
                "Cannot call showModal() on a dialog that is not connected.".into(),
            )));
        }

        // TODO: Step 5. If subject is in the popover showing state, then throw an "InvalidStateError" DOMException.

        // Step 6. If the result of firing an event named beforetoggle, using ToggleEvent, with the cancelable attribute initialized to true, the oldState attribute initialized to "closed", the newState attribute initialized to "open", and the source attribute initialized to source at subject is false, then return.
        let event = ToggleEvent::new(
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            DOMString::from("closed"),
            DOMString::from("open"),
            source.borrow().clone(),
            can_gc,
        );
        let event = event.upcast::<Event>();
        if !event.fire(self.upcast::<EventTarget>(), can_gc) {
            return Ok(());
        }

        // Step 7. If subject has an open attribute, then return.
        if subject.has_attribute(&local_name!("open")) {
            return Ok(());
        }

        // Step 8. If subject is not connected, then return.
        if !subject.is_connected() {
            return Ok(());
        }

        // TODO: Step 9. If subject is in the popover showing state, then return.

        // Step 10. Queue a dialog toggle event task given subject, "closed", "open", and source.
        self.queue_dialog_toggle_event_task("closed", "open", source);

        // Step 11. Add an open attribute to subject, whose value is the empty string.
        subject.set_bool_attribute(&local_name!("open"), true, can_gc);
        subject.set_open_state(true);

        // TODO: Step 12. Assert: subject's close watcher is not null.

        // Step 13. Set is modal of subject to true.
        self.upcast::<Element>().set_modal_state(true);

        // TODO: Step 14. Set subject's node document to be blocked by the modal dialog subject.

        // TODO: Step 15. If subject's node document's top layer does not already contain subject, then add an element to the top layer given subject.

        // TODO: Step 16. Set subject's previously focused element to the focused element.

        // TODO: Step 17. Let document be subject's node document.

        // TODO: Step 18. Let hideUntil be the result of running topmost popover ancestor given subject, document's showing hint popover list, null, and false.

        // TODO: Step 19. If hideUntil is null, then set hideUntil to the result of running topmost popover ancestor given subject, document's showing auto popover list, null, and false.

        // TODO: Step 20. If hideUntil is null, then set hideUntil to document.

        // TODO: Step 21. Run hide all popovers until given hideUntil, false, and true.

        // TODO(Issue #32702): Step 22. Run the dialog focusing steps given subject.
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-dialog-close-watcher>
    pub fn set_the_close_watcher(&self) {
        // Step 1. Assert: dialog's close watcher is null.
        assert!(self.close_watcher.borrow().is_none());
        // Step 2. Assert: dialog has an open attribute and dialog's node document is fully active.
        let dialog = self.upcast::<Element>();
        assert!(
            dialog.has_attribute(&local_name!("open")) && dialog.owner_document().is_fully_active()
        );
        // Step 3. Set dialog's close watcher to the result of establishing a close watcher given dialog's relevant global object, with:
        // - cancelAction given canPreventClose being to return the result of firing an event named cancel at dialog, with the cancelable attribute initialized to canPreventClose.
        // - closeAction being to close the dialog given dialog, dialog's request close return value, and dialog's request close source element.
        // - getEnabledState being to return true if dialog's enable close watcher for request close is true or dialog's computed closed-by state is not None; otherwise false.
        let close_watcher = InternalCloseWatcher::establish(
            dialog.owner_document().window(),
            InternalCloseWatcherOwner::Dialog(Dom::from_ref(&*self)),
        );
        self.close_watcher.borrow_mut().replace(close_watcher);
    }

    /// <https://html.spec.whatwg.org/multipage/#close-the-dialog>
    pub fn close_the_dialog(
        &self,
        result: Option<DOMString>,
        source: Option<DomRoot<Element>>,
        can_gc: CanGc,
    ) {
        let subject = self.upcast::<Element>();
        // Step 1. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 2. Fire an event named beforetoggle, using ToggleEvent, with the oldState attribute initialized to "open", the newState attribute initialized to "closed", and the source attribute initialized to source at subject.
        let event = ToggleEvent::new(
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            DOMString::from("open"),
            DOMString::from("closed"),
            source.borrow().clone(),
            can_gc,
        );
        let event = event.upcast::<Event>();
        event.fire(self.upcast::<EventTarget>(), can_gc);

        // Step 3. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 4. Queue a dialog toggle event task given subject, "open", "closed", and source.
        self.queue_dialog_toggle_event_task("open", "closed", source);

        // Step 5. Remove subject's open attribute.
        subject.remove_attribute(&ns!(), &local_name!("open"), can_gc);
        subject.set_open_state(false);

        // TODO: Step 6. If is modal of subject is true, then request an element to be removed from the top layer given subject.

        // TODO: Step 7. Let wasModal be the value of subject's is modal flag.

        // Step 8. Set is modal of subject to false.
        self.upcast::<Element>().set_modal_state(false);

        // Step 9. If result is not null, then set subject's returnValue attribute to result.
        if let Some(new_value) = result {
            *self.return_value.borrow_mut() = new_value;
        }

        // TODO: Step 10. Set subject's request close return value to null.

        // TODO: Step 11. Set subject's request close source element to null.

        // TODO: Step 12. If subject's previously focused element is not null, then:
        // TODO: Step 12.1. Let element be subject's previously focused element.
        // TODO: Step 12.2. Set subject's previously focused element to null.
        // TODO: Step 12.3. If subject's node document's focused area of the document's DOM anchor is a shadow-including inclusive descendant of subject, or wasModal is true, then run the focusing steps for element; the viewport should not be scrolled by doing this step.

        // Step 13. Queue an element task on the user interaction task source given the subject element to fire an event named close at subject.
        let target = self.upcast::<EventTarget>();
        self.owner_global()
            .task_manager()
            .user_interaction_task_source()
            .queue_simple_event(target, atom!("close"));
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-request-close>
    pub fn request_to_close(
        &self,
        return_value: Option<DOMString>,
        source: Option<DomRoot<Element>>,
        can_gc: CanGc,
    ) {
        let subject = self.upcast::<Element>();
        // Step 1. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 2. If subject is not connected or subject's node document is not fully active, then return.
        if !subject.is_connected() || !subject.owner_document().is_fully_active() {
            return;
        }

        // Step 3. Assert: subject's close watcher is not null.
        assert!(self.close_watcher.borrow().is_some());

        // Step 4. Set subject's enable close watcher for request close to true.
        self.enable_close_watcher_for_request_close.set(true);

        // Step 5. Set subject's request close return value to returnValue.
        *self.request_close_return_value.borrow_mut() = return_value;

        // Step 6. Set subject's request close source element to source.
        *self.request_close_source_element.borrow_mut() = source;

        // Step 7. Request to close subject's close watcher with false.
        self.is_running_request_close.set(true);
        self.close_watcher
            .borrow()
            .as_ref()
            .unwrap()
            .request_to_close(false, can_gc);
        self.is_running_request_close.set(false);
        // This is needed because the dialog_cleanup_steps requires a borrow of close_watcher,
        // so it can't run as part of request_to_close.
        if !subject.has_attribute(&local_name!("open")) {
            self.dialog_cleanup_steps();
        }

        // Step 8. Set subject's enable close watcher for request close to false.
        self.enable_close_watcher_for_request_close.set(false);
    }

    /// <https://html.spec.whatwg.org/multipage/#queue-a-dialog-toggle-event-task>
    pub fn queue_dialog_toggle_event_task(
        &self,
        old_state: &str,
        new_state: &str,
        source: Option<DomRoot<Element>>,
    ) {
        // TODO: Step 1. If element's dialog toggle task tracker is not null, then:
        // TODO: Step 1.1. Set oldState to element's dialog toggle task tracker's old state.
        // TODO: Step 1.2. Remove element's dialog toggle task tracker's task from its task queue.
        // TODO: Step 1.3. Set element's dialog toggle task tracker to null.
        // Step 2. Queue an element task given the DOM manipulation task source and element to run the following steps:
        let this = Trusted::new(self);
        let old_state = old_state.to_string();
        let new_state = new_state.to_string();

        let trusted_source = source
            .as_ref()
            .map(|el| Trusted::new(el.upcast::<EventTarget>()));

        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_toggle_event: move || {
                let this = this.root();

                let source = trusted_source.as_ref().map(|s| {
                    DomRoot::from_ref(s.root().downcast::<Element>().unwrap())
                });

                // Step 2.1. Fire an event named toggle at element, using ToggleEvent, with the oldState attribute initialized to oldState, the newState attribute initialized to newState, and the source attribute initialized to source.
                let event = ToggleEvent::new(
                    &this.owner_window(),
                    atom!("toggle"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    DOMString::from(old_state),
                    DOMString::from(new_state),
                    source,
                    CanGc::note(),
                );
                let event = event.upcast::<Event>();
                event.fire(this.upcast::<EventTarget>(), CanGc::note());

                // TODO: Step 2.2. Set element's dialog toggle task tracker to null.
            }));
        // TODO: Step 3. Set element's dialog toggle task tracker to a struct with task set to the just-queued task and old state set to oldState.
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-setup-steps>
    pub fn dialog_setup_steps(&self) {
        // Step 1. Assert: subject has an open attribute.
        let subject = self.upcast::<Element>();
        assert!(subject.has_attribute(&local_name!("open")));

        // Step 2. Assert: subject is connected.
        assert!(subject.is_connected());

        // TODO Step 3. Assert: subject's node document's open dialogs list does not contain subject.
        // TODO Step 4. Add subject to subject's node document's open dialogs list.

        // Step 5. Set the dialog close watcher with subject.
        self.set_the_close_watcher();
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-setup-steps>
    pub fn dialog_cleanup_steps(&self) {
        if self.is_running_request_close.get() {
            return;
        }
        // TODO Step 1. Remove subject from subject's node document's open dialogs list.

        // Step 2. If subject's close watcher is not null, then:
        // Step 2.1. Destroy subject's close watcher.
        // Step 2.2. Set subject's close watcher to null.
        if let Some(close_watcher) = self.close_watcher.borrow_mut().take() {
            close_watcher.destroy();
        }
    }

    pub(crate) fn close_watcher_close(&self, can_gc: CanGc) {
        // closeAction being to close the dialog given dialog, dialog's request close return value,
        // and dialog's request close source element.
        self.close_the_dialog(
            self.request_close_return_value.borrow().clone(),
            self.request_close_source_element.borrow().clone(),
            can_gc,
        )
    }

    pub(crate) fn close_watcher_enabled_state(&self) -> bool {
        // getEnabledState being to return true if dialog's enable close watcher for request close
        // is true or TODO dialog's computed closed-by state is not None; otherwise false.
        self.enable_close_watcher_for_request_close.get() ||
            self.upcast::<Element>()
                .state()
                .contains(ElementState::MODAL)
    }
}

impl HTMLDialogElementMethods<crate::DomTypeHolder> for HTMLDialogElement {
    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_setter!(SetOpen, "open");

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue>
    fn ReturnValue(&self) -> DOMString {
        let return_value = self.return_value.borrow();
        return_value.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue>
    fn SetReturnValue(&self, return_value: DOMString) {
        *self.return_value.borrow_mut() = return_value;
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-show>
    fn Show(&self, can_gc: CanGc) -> ErrorResult {
        let element = self.upcast::<Element>();
        // Step 1. If this has an open attribute and is modal of this is false, then return.
        if element.has_attribute(&local_name!("open")) &&
            !element.state().contains(ElementState::MODAL)
        {
            return Ok(());
        }

        // Step 2. If this has an open attribute, then throw an "InvalidStateError" DOMException.
        if element.has_attribute(&local_name!("open")) {
            return Err(Error::InvalidState(Some(
                "Cannot call show() on an already open dialog.".into(),
            )));
        }

        // Step 3. If the result of firing an event named beforetoggle, using ToggleEvent, with the cancelable attribute initialized to true, the oldState attribute initialized to "closed", and the newState attribute initialized to "open" at this is false, then return.
        let event = ToggleEvent::new(
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            DOMString::from("closed"),
            DOMString::from("open"),
            None,
            can_gc,
        );
        let event = event.upcast::<Event>();
        if !event.fire(self.upcast::<EventTarget>(), can_gc) {
            return Ok(());
        }

        // Step 4. If this has an open attribute, then return.
        if element.has_attribute(&local_name!("open")) {
            return Ok(());
        }

        // Step 5. Queue a dialog toggle event task given this, "closed", "open", and null.
        self.queue_dialog_toggle_event_task("closed", "open", None);

        // Step 6. Add an open attribute to this, whose value is the empty string.
        element.set_bool_attribute(&local_name!("open"), true, can_gc);
        element.set_open_state(true);

        // TODO: Step 7. Set this's previously focused element to the focused element.

        // TODO: Step 8. Let document be this's node document.

        // TODO: Step 9. Let hideUntil be the result of running topmost popover ancestor given this, document's showing hint popover list, null, and false.

        // TODO: Step 10. If hideUntil is null, then set hideUntil to the result of running topmost popover ancestor given this, document's showing auto popover list, null, and false.

        // TODO: Step 11. If hideUntil is null, then set hideUntil to document.

        // TODO: Step 12. Run hide all popovers until given hideUntil, false, and true.

        // TODO(Issue #32702): Step 13. Run the dialog focusing steps given this.

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-showmodal>
    fn ShowModal(&self, can_gc: CanGc) -> ErrorResult {
        // The showModal() method steps are to show a modal dialog given this and null.
        self.show_a_modal(None, can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-close>
    fn Close(&self, return_value: Option<DOMString>, can_gc: CanGc) {
        // Step 1. If returnValue is not given, then set it to null.
        // Step 2. Close the dialog this with returnValue and null.
        self.close_the_dialog(return_value, None, can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-requestclose>
    fn RequestClose(&self, return_value: Option<DOMString>, can_gc: CanGc) {
        // Step 1. If returnValue is not given, then set it to null.
        // Step 2. Request to close the dialog this with returnValue and null.
        self.request_to_close(return_value, None, can_gc);
    }
}

impl VirtualMethods for HTMLDialogElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-dialog-element:is-valid-command-steps>
    fn is_valid_command_steps(&self, command: CommandState) -> bool {
        // Step 1. If command is in the Close state, the Request Close state, or the
        // ShowModal state, then return true.
        if command == CommandState::Close ||
            command == CommandState::RequestClose ||
            command == CommandState::ShowModal
        {
            return true;
        }
        // Step 2. Return false.
        false
    }

    /// <https://html.spec.whatwg.org/multipage/#the-dialog-element:command-steps>
    fn command_steps(
        &self,
        source: DomRoot<HTMLButtonElement>,
        command: CommandState,
        can_gc: CanGc,
    ) -> bool {
        if self
            .super_type()
            .unwrap()
            .command_steps(source.clone(), command, can_gc)
        {
            return true;
        }

        // TODO Step 1. If element is in the popover showing state, then return.
        let element = self.upcast::<Element>();

        // Step 2. If command is in the Close state and element has an open attribute, then
        // close the dialog element with source's optional value and source.
        if command == CommandState::Close && element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            self.close_the_dialog(source.optional_value(), Some(button_element), can_gc);
            return true;
        }

        // Step 3. If command is in the Request Close state and element has an open attribute,
        // then request to close the dialog element with source's optional value and source.
        if command == CommandState::RequestClose && element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            let _ = self.request_to_close(source.optional_value(), Some(button_element), can_gc);
            return true;
        }

        // Step 4. If command is the Show Modal state and element does not have an open attribute,
        // then show a modal dialog given element and source.
        if command == CommandState::ShowModal && !element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            let _ = self.show_a_modal(Some(button_element), can_gc);
            return true;
        }

        false
    }

    /// <https://html.spec.whatwg.org/multipage/#the-dialog-element:html-element-insertion-steps>
    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        self.super_type().unwrap().bind_to_tree(cx, context);
        let inserted_node = self.upcast::<Element>();

        // Step 1. If insertedNode's node document is not fully active, then return.
        if !inserted_node.owner_document().is_fully_active() {
            return;
        }

        // Step 2. If insertedNode has an open attribute and is connected, then run the dialog setup steps given insertedNode.
        if inserted_node.has_attribute(&local_name!("open")) && inserted_node.is_connected() {
            self.dialog_setup_steps();
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-dialog-element:html-element-removing-steps>
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);
        let removed_node = self.upcast::<Element>();

        // Step 1. If removedNode has an open attribute, then run the dialog cleanup steps given removedNode.
        if removed_node.has_attribute(&local_name!("open")) {
            self.dialog_cleanup_steps();
        }

        // TODO Step 2. If removedNode's node document's top layer contains removedNode, then remove an element from the top layer immediately given removedNode.

        // Step 3. Set is modal of removedNode to false.
        removed_node.set_modal_state(false)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        // Step 1. If namespace is not null, then return.
        if *attr.namespace() != ns!() {
            return;
        }

        // Step 2. If localName is not open, then return.
        if attr.local_name() != &local_name!("open") {
            return;
        }

        // Step 3. If value is null and oldValue is not null, then run the dialog cleanup steps given element.
        if matches!(mutation, AttributeMutation::Removed) {
            self.dialog_cleanup_steps();
        }

        // Step 4. If element's node document is not fully active, then return.
        let element = self.upcast::<Element>();
        if !element.owner_document().is_fully_active() {
            return;
        }

        // Step 5. If element is not connected, then return.
        if !element.is_connected() {
            return;
        }

        // Step 6. If value is not null and oldValue is null, then run the dialog setup steps given element.
        if matches!(mutation, AttributeMutation::Set(..)) {
            self.dialog_setup_steps();
        }
    }
}
