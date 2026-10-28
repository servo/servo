/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElementMethods;
use script_bindings::domstring::TracedDOMString;
use script_bindings::error::{Error, ErrorResult};
use script_bindings::root::Dom;
use stylo_dom::ElementState;

use crate::dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::closewatcher::internalclosewatcher::{
    InternalCloseWatcher, InternalCloseWatcherOwner,
};
use crate::dom::document::Document;
use crate::dom::element::storage::AttrRef;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::htmlbuttonelement::{CommandState, HTMLButtonElement};
use crate::dom::iterators::ShadowIncluding;
use crate::dom::node::focus::FocusTrigger;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::toggleevent::ToggleEvent;

#[dom_struct]
pub(crate) struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DomRefCell<TracedDOMString>,
    /// <https://html.spec.whatwg.org/multipage/#dialog-close-watcher>
    #[conditional_malloc_size_of]
    close_watcher: DomRefCell<Option<Rc<InternalCloseWatcher>>>,
    /// <https://html.spec.whatwg.org/multipage/#request-close-return-value>
    request_close_return_value: DomRefCell<Option<TracedDOMString>>,
    /// <https://html.spec.whatwg.org/multipage/#request-close-source-element>
    request_close_source_element: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#enable-close-watcher-for-requestclose()>
    enable_close_watcher_for_request_close: Cell<bool>,
}

impl HTMLDialogElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDialogElement {
        HTMLDialogElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            return_value: DomRefCell::new(DOMString::new().traced()),
            close_watcher: Default::default(),
            request_close_return_value: Default::default(),
            request_close_source_element: Default::default(),
            enable_close_watcher_for_request_close: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLDialogElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLDialogElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#show-a-modal-dialog>
    pub(crate) fn show_a_modal(
        &self,
        cx: &mut js::context::JSContext,
        source: Option<DomRoot<Element>>,
    ) -> ErrorResult {
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
            cx,
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            DOMString::from_static("closed"),
            DOMString::from_static("open"),
            source.as_deref(),
        );
        let event = event.upcast::<Event>();
        if !event.fire(cx, self.upcast::<EventTarget>()) {
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
        subject.set_bool_attribute(cx, &local_name!("open"), true);
        subject.set_open_state(cx.no_gc(), true);

        // TODO: Step 12. Assert: subject's close watcher is not null.

        // Step 13. Set is modal of subject to true.
        self.upcast::<Element>().set_modal_state(cx.no_gc(), true);

        // TODO: Step 14. Set subject's node document to be blocked by the modal dialog subject.

        // TODO: Step 15. If subject's node document's top layer does not already contain subject, then add an element to the top layer given subject.

        // Step 16. Set subject's previously focused element to the focused element.
        self.upcast::<HTMLElement>().set_previously_focused_element(
            self.owner_document()
                .focus_handler()
                .focused_area()
                .element(),
            cx.no_gc(),
        );

        // TODO: Step 17. Let document be subject's node document.

        // TODO: Step 18. Let hideUntil be the result of running topmost popover ancestor given subject, document's showing hint popover list, null, and false.

        // TODO: Step 19. If hideUntil is null, then set hideUntil to the result of running topmost popover ancestor given subject, document's showing auto popover list, null, and false.

        // TODO: Step 20. If hideUntil is null, then set hideUntil to document.

        // TODO: Step 21. Run hide all popovers until given hideUntil, false, and true.

        // Step 22. Run the dialog focusing steps given subject.
        self.run_dialog_focusing_steps(cx);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-dialog-close-watcher>
    pub(crate) fn set_the_close_watcher(&self, no_gc: &NoGC) {
        // Step 1. Assert: dialog's close watcher is null.
        assert!(self.close_watcher.borrow().is_none());

        // Step 2. Assert: dialog has an open attribute and dialog's node document is fully active.
        let dialog = self.upcast::<Element>();
        assert!(
            dialog.has_attribute(&local_name!("open")) && dialog.owner_document().is_fully_active()
        );

        // Step 3. Set dialog's close watcher to the result of establishing a close watcher given
        // dialog's relevant global object, with:
        // - cancelAction given canPreventClose being to return the result of firing an event named
        //   cancel at dialog, with the cancelable attribute initialized to canPreventClose.
        // - closeAction being to close the dialog given dialog, dialog's request close return
        //   value, and dialog's request close source element.
        // - getEnabledState being to return true if dialog's enable close watcher for request close
        //   is true or dialog's computed closed-by state is not None; otherwise false.
        let close_watcher = InternalCloseWatcher::establish(
            no_gc,
            dialog.owner_document().window(),
            InternalCloseWatcherOwner::Dialog(Dom::from_ref(self)),
        );
        self.close_watcher
            .safe_borrow_mut(no_gc)
            .replace(close_watcher);
    }

    /// <https://html.spec.whatwg.org/multipage/#close-the-dialog>
    pub(crate) fn close_the_dialog(
        &self,
        cx: &mut js::context::JSContext,
        result: Option<DOMString>,
        source: Option<DomRoot<Element>>,
    ) {
        let subject = self.upcast::<Element>();
        // Step 1. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 2. Fire an event named beforetoggle, using ToggleEvent, with the oldState attribute initialized to "open", the newState attribute initialized to "closed", and the source attribute initialized to source at subject.
        let event = ToggleEvent::new(
            cx,
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
            DOMString::from_static("open"),
            DOMString::from_static("closed"),
            source.as_deref(),
        );
        let event = event.upcast::<Event>();
        event.fire(cx, self.upcast::<EventTarget>());

        // Step 3. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 4. Queue a dialog toggle event task given subject, "open", "closed", and source.
        self.queue_dialog_toggle_event_task("open", "closed", source);

        // Step 5. Remove subject's open attribute.
        subject.remove_attribute(cx, &ns!(), &local_name!("open"));
        subject.set_open_state(cx.no_gc(), false);

        // TODO: Step 6. If is modal of subject is true, then request an element to be removed from the top layer given subject.

        // Step 7. Let wasModal be the value of subject's is modal flag.
        let was_modal = subject.state().contains(ElementState::MODAL);

        // Step 8. Set is modal of subject to false.
        self.upcast::<Element>().set_modal_state(cx.no_gc(), false);

        // Step 9. If result is not null, then set subject's returnValue attribute to result.
        if let Some(new_value) = result {
            *self.return_value.safe_borrow_mut(cx.no_gc()) = new_value.traced();
        }

        // Step 10. Set subject's request close return value to null.
        *self.request_close_return_value.safe_borrow_mut(cx) = None;

        // Step 11. Set subject's request close source element to null.
        self.request_close_source_element.set(None);

        // Step 12. If subject's previously focused element is not null, then:
        if let Some(element) = self
            .upcast::<HTMLElement>()
            .previously_focused_element(cx.no_gc())
        {
            // Step 12.1. Let element be subject's previously focused element.
            // Step 12.2. Set subject's previously focused element to null.
            self.upcast::<HTMLElement>()
                .set_previously_focused_element(None, cx.no_gc());

            // Step 12.3. If subject's node document's focused area of the document's DOM anchor is
            // a shadow-including inclusive descendant of subject, or wasModal is true, then run the
            // focusing steps for element; the viewport should not be scrolled by doing this step.
            let subject_node = subject.upcast::<Node>();
            let document = subject.owner_document();
            if document
                .focus_handler()
                .focused_area()
                .dom_anchor(&document)
                .traverse_preorder(ShadowIncluding::Yes)
                .any(|node| &*node == subject_node) ||
                was_modal
            {
                element
                    .upcast::<Node>()
                    .run_the_focusing_steps(cx, None, FocusTrigger::Other);
            }
        }

        // Step 13. Queue an element task on the user interaction task source given the subject element to fire an event named close at subject.
        let target = self.upcast::<EventTarget>();
        self.owner_global()
            .task_manager()
            .user_interaction_task_source()
            .queue_simple_event(target, atom!("close"));
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-request-close>
    pub(crate) fn request_to_close(
        &self,
        cx: &mut JSContext,
        return_value: Option<DOMString>,
        source: Option<DomRoot<Element>>,
    ) {
        let subject = self.upcast::<Element>();

        // Step 1. If subject does not have an open attribute, then return.
        if !subject.has_attribute(&local_name!("open")) {
            return;
        }

        // Step 2. If subject is not connected or subject's node document is not fully active, then
        // return.
        if !subject.is_connected() || !subject.owner_document().is_fully_active() {
            return;
        }

        // Step 3. Assert: subject's close watcher is not null.
        //
        // Clone the handle so the cell isn't borrowed while script runs: a `close` listener can
        // call `dialog.close()`, which runs `dialog_cleanup_steps` and nulls the close watcher.
        let close_watcher = self
            .close_watcher
            .borrow()
            .clone()
            .expect("Close watcher must always exist");

        // Step 4. Set subject's enable close watcher for request close to true.
        self.enable_close_watcher_for_request_close.set(true);

        // Step 5. Set subject's request close return value to returnValue.
        *self.request_close_return_value.safe_borrow_mut(cx) = return_value.map(DOMString::traced);

        // Step 6. Set subject's request close source element to source.
        self.request_close_source_element.set(source.as_deref());

        // Step 7. Request to close subject's close watcher with false.
        close_watcher.request_to_close(cx, false);

        // Step 8. Set subject's enable close watcher for request close to false.
        self.enable_close_watcher_for_request_close.set(false);
    }

    /// <https://html.spec.whatwg.org/multipage/#queue-a-dialog-toggle-event-task>
    pub(crate) fn queue_dialog_toggle_event_task(
        &self,
        old_state: &'static str,
        new_state: &'static str,
        source: Option<DomRoot<Element>>,
    ) {
        // TODO: Step 1. If element's dialog toggle task tracker is not null, then:
        // TODO: Step 1.1. Set oldState to element's dialog toggle task tracker's old state.
        // TODO: Step 1.2. Remove element's dialog toggle task tracker's task from its task queue.
        // TODO: Step 1.3. Set element's dialog toggle task tracker to null.
        // Step 2. Queue an element task given the DOM manipulation task source and element to run the following steps:
        let this = Trusted::new(self);
        let trusted_source = source.map(|el| Trusted::new(&*el));

        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_toggle_event: move |cx| {
                let this = this.root();

                let source = trusted_source.map(|s| s.root());

                // Step 2.1. Fire an event named toggle at element, using ToggleEvent, with the oldState attribute initialized to oldState, the newState attribute initialized to newState, and the source attribute initialized to source.
                let event = ToggleEvent::new(
                    cx,
                    &this.owner_window(),
                    atom!("toggle"),
                    EventBubbles::DoesNotBubble,
                    EventCancelable::NotCancelable,
                    DOMString::from_static(old_state),
                    DOMString::from_static(new_state),
                    source.as_deref(),
                );
                let event = event.upcast::<Event>();
                event.fire(cx, this.upcast::<EventTarget>());

                // TODO: Step 2.2. Set element's dialog toggle task tracker to null.
            }));
        // TODO: Step 3. Set element's dialog toggle task tracker to a struct with task set to the just-queued task and old state set to oldState.
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-focusing-steps>
    fn run_dialog_focusing_steps(&self, cx: &mut JSContext) {
        // TODO: Step 1. If the allow focus steps given subject's node document return false, then return.

        // Step 2. Let control be null.
        rooted!(&in(cx) let mut control = None);

        // Step 3. If subject has the autofocus attribute, then set control to subject.
        if self.upcast::<HTMLElement>().Autofocus() {
            control.set(
                self.upcast::<Node>()
                    .get_the_focusable_area(cx, FocusTrigger::Other),
            );
        }

        // Step 4. If control is null, then set control to the focus delegate of subject.
        if control.is_none() {
            control.set(self.upcast::<Node>().focus_delegate(cx, None));
        }

        // Step 5. If control is null, then set control to subject.
        if control.is_none() {
            control.set(
                self.upcast::<Node>()
                    .get_the_focusable_area(cx, FocusTrigger::Other),
            );
        }

        // Step 6. Run the focusing steps for control.
        // FIXME: Use the focusing step once they support a focusable area as an argument
        if control.is_some() {
            let document = self.owner_document();
            document.focus_handler().focus(cx, &control.take().unwrap());
        }

        // TODO: Step 7. Let topDocument be control's node navigable's top-level traversable's active document.
        // TODO: Step 8. If control's node document's origin is not the same as the origin of topDocument, then return.
        // TODO: Step 9. Empty topDocument's autofocus candidates.
        // TODO: Step 10. Set topDocument's autofocus processed flag to true.
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-setup-steps>
    pub(crate) fn dialog_setup_steps(&self, no_gc: &NoGC) {
        // Step 1. Assert: subject has an open attribute.
        let subject = self.upcast::<Element>();
        assert!(subject.has_attribute(&local_name!("open")));

        // Step 2. Assert: subject is connected.
        assert!(subject.is_connected());

        // TODO Step 3. Assert: subject's node document's open dialogs list does not contain
        // subject.
        // TODO Step 4. Add subject to subject's node document's open dialogs list.

        // Step 5. Set the dialog close watcher with subject.
        self.set_the_close_watcher(no_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#dialog-setup-steps>
    pub(crate) fn dialog_cleanup_steps(&self, no_gc: &NoGC) {
        // TODO Step 1. Remove subject from subject's node document's open dialogs list.

        // Step 2. If subject's close watcher is not null, then:
        // Step 2.1. Destroy subject's close watcher.
        // Step 2.2. Set subject's close watcher to null.
        let close_watcher = self.close_watcher.safe_borrow_mut(no_gc).take();
        if let Some(close_watcher) = close_watcher {
            close_watcher.destroy(no_gc);
        }
    }

    pub(crate) fn close_watcher_close(&self, cx: &mut JSContext) {
        let return_value = self
            .request_close_return_value
            .borrow()
            .clone()
            .map(TracedDOMString::root);
        let source = self.request_close_source_element.get();
        self.close_the_dialog(cx, return_value, source)
    }

    pub(crate) fn close_watcher_enabled_state(&self) -> bool {
        self.enable_close_watcher_for_request_close.get() ||
            self.computed_closed_by_state() != ComputedClosedByState::None
    }

    /// <https://html.spec.whatwg.org/multipage/#computed-closed-by-state>
    fn computed_closed_by_state(&self) -> ComputedClosedByState {
        let element = self.upcast::<Element>();

        // Step 2. Return the state of dialog's closedby attribute.
        if let Some(closed_by) = element.get_attribute_string_ref(&local_name!("closedby")) {
            if closed_by.eq_ignore_ascii_case("any") {
                return ComputedClosedByState::Any;
            }
            if closed_by.eq_ignore_ascii_case("closerequest") {
                return ComputedClosedByState::CloseRequest;
            }
            if closed_by.eq_ignore_ascii_case("none") {
                return ComputedClosedByState::None;
            }
        }

        // Step 1. If the state of dialog's closedby attribute is Auto:
        // https://html.spec.whatwg.org/multipage/#attr-dialog-closedby-auto-state
        // > The closedby attribute's invalid value default and missing value default are both the
        // > Auto state.

        // Step 1.1. If dialog's is modal is true, then return Close Request.
        if element.state().contains(ElementState::MODAL) {
            return ComputedClosedByState::CloseRequest;
        }

        // Step 1.2. Return None.
        ComputedClosedByState::None
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
        return_value.clone().root()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue>
    fn SetReturnValue(&self, cx: &mut JSContext, return_value: DOMString) {
        *self.return_value.safe_borrow_mut(cx.no_gc()) = return_value.traced();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-closedby>
    fn ClosedBy(&self) -> DOMString {
        // > The closedBy getter steps are to return the keyword
        // > corresponding to the computed closed-by state given this.
        match self.computed_closed_by_state() {
            ComputedClosedByState::Any => DOMString::from_static("any"),
            ComputedClosedByState::CloseRequest => DOMString::from_static("closerequest"),
            ComputedClosedByState::None => DOMString::from_static("none"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#the-dialog-element:dom-dialog-closedby
    make_setter!(SetClosedBy, "closedby");

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-show>
    fn Show(&self, cx: &mut js::context::JSContext) -> ErrorResult {
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
            cx,
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            DOMString::from_static("closed"),
            DOMString::from_static("open"),
            None,
        );
        let event = event.upcast::<Event>();
        if !event.fire(cx, self.upcast::<EventTarget>()) {
            return Ok(());
        }

        // Step 4. If this has an open attribute, then return.
        if element.has_attribute(&local_name!("open")) {
            return Ok(());
        }

        // Step 5. Queue a dialog toggle event task given this, "closed", "open", and null.
        self.queue_dialog_toggle_event_task("closed", "open", None);

        // Step 6. Add an open attribute to this, whose value is the empty string.
        element.set_bool_attribute(cx, &local_name!("open"), true);
        element.set_open_state(cx.no_gc(), true);

        // Step 7. Set this's previously focused element to the focused element.
        self.upcast::<HTMLElement>().set_previously_focused_element(
            self.owner_document()
                .focus_handler()
                .focused_area()
                .element(),
            cx.no_gc(),
        );

        // TODO: Step 8. Let document be this's node document.

        // TODO: Step 9. Let hideUntil be the result of running topmost popover ancestor given this, document's showing hint popover list, null, and false.

        // TODO: Step 10. If hideUntil is null, then set hideUntil to the result of running topmost popover ancestor given this, document's showing auto popover list, null, and false.

        // TODO: Step 11. If hideUntil is null, then set hideUntil to document.

        // TODO: Step 12. Run hide all popovers until given hideUntil, false, and true.

        // Step 13. Run the dialog focusing steps given this.
        self.run_dialog_focusing_steps(cx);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-showmodal>
    fn ShowModal(&self, cx: &mut js::context::JSContext) -> ErrorResult {
        // The showModal() method steps are to show a modal dialog given this and null.
        self.show_a_modal(cx, None)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-close>
    fn Close(&self, cx: &mut js::context::JSContext, return_value: Option<DOMString>) {
        // Step 1. If returnValue is not given, then set it to null.
        // Step 2. Close the dialog this with returnValue and null.
        self.close_the_dialog(cx, return_value, None);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-requestclose>
    fn RequestClose(&self, cx: &mut js::context::JSContext, return_value: Option<DOMString>) {
        // Step 1. If returnValue is not given, then set it to null.
        // Step 2. Request to close the dialog this with returnValue and null.
        self.request_to_close(cx, return_value, None);
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
        cx: &mut js::context::JSContext,
        source: DomRoot<HTMLButtonElement>,
        command: CommandState,
    ) -> bool {
        if self
            .super_type()
            .unwrap()
            .command_steps(cx, source.clone(), command)
        {
            return true;
        }

        // TODO Step 1. If element is in the popover showing state, then return.
        let element = self.upcast::<Element>();

        // Step 2. If command is in the Close state and element has an open attribute, then
        // close the dialog element with source's optional value and source.
        if command == CommandState::Close && element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            self.close_the_dialog(cx, source.optional_value(), Some(button_element));
            return true;
        }

        // Step 3. If command is in the Request Close state and element has an open attribute,
        // then request to close the dialog element with source's optional value and source.
        if command == CommandState::RequestClose && element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            self.request_to_close(cx, source.optional_value(), Some(button_element));
            return true;
        }

        // Step 4. If command is the Show Modal state and element does not have an open attribute,
        // then show a modal dialog given element and source.
        if command == CommandState::ShowModal && !element.has_attribute(&local_name!("open")) {
            let button_element = DomRoot::from_ref(source.upcast::<Element>());
            let _ = self.show_a_modal(cx, Some(button_element));
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

        // Step 2. If insertedNode has an open attribute and is connected, then run the dialog setup
        // steps given insertedNode.
        if inserted_node.has_attribute(&local_name!("open")) && inserted_node.is_connected() {
            self.dialog_setup_steps(cx.no_gc());
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-dialog-element:html-element-removing-steps>
    fn unbind_from_tree(&self, cx: &mut js::context::JSContext, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(cx, context);
        let removed_node = self.upcast::<Element>();

        // Step 1. If removedNode has an open attribute, then run the dialog cleanup steps given
        // removedNode.
        if removed_node.has_attribute(&local_name!("open")) {
            self.dialog_cleanup_steps(cx.no_gc());
        }

        // TODO Step 2. If removedNode's node document's top layer contains removedNode, then remove
        // an element from the top layer immediately given removedNode.

        // Step 3. Set is modal of removedNode to false.
        removed_node.set_modal_state(cx.no_gc(), false)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);

        // Step 1. If namespace is not null, then return.
        if *attr.namespace() != ns!() {
            return;
        }

        // Step 2. If localName is not open, then return.
        if attr.local_name() != &local_name!("open") {
            return;
        }

        // Step 3. If value is null and oldValue is not null, then run the dialog cleanup steps
        // given element.
        if matches!(mutation, AttributeMutation::Removed) {
            self.dialog_cleanup_steps(cx.no_gc());
            return;
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

        // Step 6. If value is not null and oldValue is null, then run the dialog setup steps given
        // element.
        if matches!(mutation, AttributeMutation::Set(..)) {
            self.dialog_setup_steps(cx.no_gc());
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ComputedClosedByState {
    Any,
    CloseRequest,
    None,
}
