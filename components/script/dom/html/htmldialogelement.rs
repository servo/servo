/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::borrow::Borrow;
use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use script_bindings::error::{Error, ErrorResult};

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::toggleevent::ToggleEvent;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DomRefCell<DOMString>,
    is_modal: Cell<bool>,
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
            is_modal: Cell::new(false),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
        if subject.has_attribute(&local_name!("open")) && self.is_modal.get() {
            return Ok(());
        }

        // Step 2. If subject has an open attribute, then throw an "InvalidStateError" DOMException.
        if subject.has_attribute(&local_name!("open")) {
            return Err(Error::InvalidState(None));
        }

        // Step 3. If subject's node document is not fully active, then throw an "InvalidStateError" DOMException.
        if !subject.owner_document().is_fully_active() {
            return Err(Error::InvalidState(None));
        }

        // Step 4. If subject is not connected, then throw an "InvalidStateError" DOMException.
        if !subject.is_connected() {
            return Err(Error::InvalidState(None));
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
        self.queue_dialog_toggle_event_task(
            DOMString::from("closed"),
            DOMString::from("open"),
            source,
        );

        // Step 11. Add an open attribute to subject, whose value is the empty string.
        subject.set_bool_attribute(&local_name!("open"), true, can_gc);

        // TODO: Step 12. Assert: subject's close watcher is not null.

        // Step 13. Set is modal of subject to true.
        self.is_modal.set(true);

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
        self.queue_dialog_toggle_event_task(
            DOMString::from("open"),
            DOMString::from("closed"),
            source,
        );

        // Step 5. Remove subject's open attribute.
        subject.remove_attribute(&ns!(), &local_name!("open"), can_gc);

        // TODO: Step 6. If is modal of subject is true, then request an element to be removed from the top layer given subject.

        // TODO: Step 7. Let wasModal be the value of subject's is modal flag.

        // Step 8. Set is modal of subject to false.
        self.is_modal.set(false);

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

    /// <https://html.spec.whatwg.org/multipage/#queue-a-dialog-toggle-event-task>
    pub fn queue_dialog_toggle_event_task(
        &self,
        old_state: DOMString,
        new_state: DOMString,
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
        if element.has_attribute(&local_name!("open")) && !self.is_modal.get() {
            return Ok(());
        }

        // Step 2. If this has an open attribute, then throw an "InvalidStateError" DOMException.
        if element.has_attribute(&local_name!("open")) {
            return Err(Error::InvalidState(None));
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
        self.queue_dialog_toggle_event_task(
            DOMString::from("closed"),
            DOMString::from("open"),
            None,
        );

        // Step 6. Add an open attribute to this, whose value is the empty string.
        element.set_bool_attribute(&local_name!("open"), true, can_gc);

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
}
