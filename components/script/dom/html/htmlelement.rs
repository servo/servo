/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, QualName, local_name, ns};
use js::context::{JSContext, NoGC};
use js::rust::HandleObject;
use layout_api::{QueryMsg, ScrollContainerQueryFlags, ScrollContainerResponse};
use rustc_hash::FxHashSet;
use script_bindings::callback::RootedCallback;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::DocumentFragmentBinding::DocumentFragmentMethods;
use script_bindings::codegen::GenericBindings::ElementBinding::ScrollLogicalPosition;
use script_bindings::codegen::GenericBindings::WindowBinding::ScrollBehavior;
use script_bindings::dom::UnrootedDom;
use style::attr::AttrValue;
use stylo_dom::ElementState;

use crate::dom::NodeFlags;
use crate::dom::activation::Activatable;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterData_Binding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::EventHandlerBinding::{
    EventHandlerNonNull, OnErrorEventHandlerNonNull,
};
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::{
    HTMLElementMethods, ShowPopoverOptions,
};
use crate::dom::bindings::codegen::Bindings::HTMLLabelElementBinding::HTMLLabelElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOrSVGOrMathMLElementBinding::FocusOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::TogglePopoverOptionsOrBoolean;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::closewatcher::internalclosewatcher::{
    InternalCloseWatcher, InternalCloseWatcherOwner,
};
use crate::dom::css::cssstyledeclaration::{
    CSSModificationAccess, CSSStyleDeclaration, CSSStyleOwner,
};
use crate::dom::customelementregistry::{CustomElementRegistry, CustomElementState};
use crate::dom::document::Document;
use crate::dom::document::focus::FocusableArea;
use crate::dom::document_event_handler::character_to_code;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::domstringmap::DOMStringMap;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::element::{
    AttributeMutation, CustomElementCreationMode, Element, ElementCreator,
    is_element_affected_by_legacy_background_presentational_hint,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::form_controls::htmlbuttonelement::HTMLButtonElement;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::form_controls::input_type::InputType;
use crate::dom::html::htmlbodyelement::HTMLBodyElement;
use crate::dom::html::htmldetailselement::HTMLDetailsElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::html::htmlframesetelement::HTMLFrameSetElement;
use crate::dom::html::htmlhtmlelement::HTMLHtmlElement;
use crate::dom::html::htmllabelelement::HTMLLabelElement;
use crate::dom::html::htmltextareaelement::HTMLTextAreaElement;
use crate::dom::html::interactive::htmldialogelement::HTMLDialogElement;
use crate::dom::html::internals::elementinternals::ElementInternals;
use crate::dom::htmlelement::PopoverState::NoPopover;
use crate::dom::htmlformelement::FormControlElementHelpers;
use crate::dom::iterators::ShadowIncluding;
use crate::dom::medialist::MediaList;
use crate::dom::node::focus::FocusTrigger;
use crate::dom::node::virtualmethods::VirtualMethods;
use crate::dom::node::{
    BindContext, MoveContext, Node, NodeDamage, NodeTraits, UnbindContext,
    from_untrusted_node_address,
};
use crate::dom::shadowroot::shadowroot::ShadowRoot;
use crate::dom::text::Text;
use crate::dom::toggleevent::ToggleEvent;
use crate::dom::window::scrolling_box::{ScrollAxisState, ScrollRequirement};

/// <https://html.spec.whatwg.org/multipage/#attr-popover>
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq)]
pub(crate) enum PopoverState {
    Auto,
    Manual,
    Hint,
    NoPopover,
}

impl PopoverState {
    /// <https://html.spec.whatwg.org/multipage/#the-popover-attribute:attr-popover-4>
    fn from_attribute_value(value: Option<String>) -> Self {
        // > The popover attribute is an enumerated attribute with the following keywords and states:
        // > - auto: Auto
        // > - manual: Manual
        // > - hint: Hint
        // > The attribute's missing value default is the No Popover state, its invalid value
        // > default is the Manual state, and its empty value default is the Auto state.
        let Some(value) = value else {
            return PopoverState::NoPopover;
        };

        if value.is_empty() || value.eq_ignore_ascii_case("auto") {
            PopoverState::Auto
        } else if value.eq_ignore_ascii_case("hint") {
            PopoverState::Hint
        } else {
            PopoverState::Manual
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-popover>
    fn reflect_attribute(&self) -> Option<DOMString> {
        // > The popover IDL attribute must reflect the popover attribute, limited to only known values.
        match self {
            PopoverState::Auto => Some(DOMString::from_static("auto")),
            PopoverState::Manual => Some(DOMString::from_static("manual")),
            PopoverState::Hint => Some(DOMString::from_static("hint")),
            PopoverState::NoPopover => None,
        }
    }
}

#[dom_struct]
pub(crate) struct HTMLElement {
    element: Element,
    style_decl: MutNullableDom<CSSStyleDeclaration>,
    dataset: MutNullableDom<DOMStringMap>,
}

impl HTMLElement {
    pub(crate) fn new_inherited(
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLElement {
        HTMLElement::new_inherited_with_state(ElementState::empty(), tag_name, prefix, document)
    }

    pub(crate) fn new_inherited_with_state(
        state: ElementState,
        tag_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLElement {
        HTMLElement {
            element: Element::new_inherited_with_state(
                state,
                tag_name,
                ns!(html),
                prefix,
                document,
            ),
            style_decl: Default::default(),
            dataset: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
        )
    }

    fn is_body_or_frameset(&self) -> bool {
        let eventtarget = self.upcast::<EventTarget>();
        eventtarget.is::<HTMLBodyElement>() || eventtarget.is::<HTMLFrameSetElement>()
    }

    /// Calls into the layout engine to generate a plain text representation
    /// of a [`HTMLElement`] as specified when getting the `.innerText` or
    /// `.outerText` in JavaScript.`
    ///
    /// <https://html.spec.whatwg.org/multipage/#get-the-text-steps>
    pub(crate) fn get_inner_outer_text(&self, cx: &mut JSContext) -> DOMString {
        let node = self.upcast::<Node>();
        let window = node.owner_window();
        let element = self.as_element();

        // Step 1.
        let element_not_rendered = !node.is_connected() || !element.has_css_layout_box(cx);
        if element_not_rendered {
            return node.GetTextContent().unwrap();
        }

        window.layout_reflow(cx, QueryMsg::ElementInnerOuterTextQuery);
        let text = window
            .layout()
            .query_element_inner_outer_text(node.to_trusted_node_address());

        DOMString::from(text)
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-inner-text-steps>
    pub(crate) fn set_inner_text(&self, cx: &mut JSContext, input: DOMString) {
        // Step 1: Let fragment be the rendered text fragment for value given element's node
        // document.
        let fragment = self.rendered_text_fragment(cx, input);

        // Step 2: Replace all with fragment within element.
        Node::replace_all(cx, Some(fragment.upcast()), self.upcast::<Node>());
    }

    /// <https://html.spec.whatwg.org/multipage/#matches-the-environment>
    pub(crate) fn media_attribute_matches_media_environment(&self) -> bool {
        // A string matches the environment of the user if it is the empty string,
        // a string consisting of only ASCII whitespace, or is a media query list that
        // matches the user's environment according to the definitions given in Media Queries. [MQ]
        self.element
            .get_attribute_string_ref(&local_name!("media"))
            .is_none_or(|media| {
                let media = media.as_attr_ref();
                MediaList::matches_environment(&self.owner_document(), &media.value())
            })
    }

    /// <https://html.spec.whatwg.org/multipage/#editing-host>
    pub(crate) fn is_editing_host(&self) -> bool {
        // > An editing host is either an HTML element with its contenteditable attribute in the true state or plaintext-only state,
        matches!(&*self.ContentEditable().str(), "true" | "plaintext-only")
        // > or a child HTML element of a Document whose design mode enabled is true.
        // TODO
    }

    pub(crate) fn previously_focused_element(&self, no_gc: &NoGC) -> Option<DomRoot<Element>> {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .previously_focused_element
            .get()
    }

    pub(crate) fn set_previously_focused_element(&self, element: Option<&Element>, no_gc: &NoGC) {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .previously_focused_element
            .set(element);
    }

    pub(crate) fn ensure_element_internals(&self, cx: &mut JSContext) -> DomRoot<ElementInternals> {
        let element = self.upcast::<Element>();
        let Some(element_internals) = element.get_element_internals() else {
            let internals = ElementInternals::new(cx, self);
            element.ensure_rare_data(cx.no_gc()).element_internals =
                Some(Dom::from_ref(&*internals));
            return internals;
        };
        element_internals
    }

    pub(crate) fn is_disableable_form_control(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement,
            )) => true,
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLElement)) => {
                self.is_form_associated_custom_element()
            },
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-fe-disabled>
    pub(crate) fn is_form_element_disabled(&self, no_gc: &NoGC) -> bool {
        // > A form control is disabled if any of the following are true:
        // > - The element is a button, input, select, textarea, or form-associated custom
        // > element, and the disabled attribute is specified on this element (regardless of
        // > its value).
        // > - The element is a descendant of a fieldset element whose disabled attribute is
        // > specified, and is not a descendant of that fieldset element's first legend element
        // > child, if any.
        let element = self.as_element();
        self.is_disableable_form_control() &&
            (element.has_attribute(&local_name!("disabled")) ||
                element.is_disabled_by_ancestor_fieldset(no_gc))
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-popover>
    pub(crate) fn popover_state(&self) -> PopoverState {
        let attribute_value = self
            .upcast::<Element>()
            .get_attribute_string_value(&local_name!("popover"));
        PopoverState::from_attribute_value(attribute_value)
    }

    /// <https://html.spec.whatwg.org/multipage/#show-popover>
    pub(crate) fn show_popover(
        &self,
        cx: &mut JSContext,
        throw_exceptions: bool,
        source: Option<&HTMLElement>,
    ) -> Fallible<()> {
        let element = self.upcast::<Element>();

        // Step 1. Let document be element's node document.
        let document = self.owner_document();
        let overlay_handler = document.overlay_handler();

        // Step 2. If document's showing popover is true or document's hiding popover nesting
        // count is not 0:
        if overlay_handler.showing_popover() || overlay_handler.hiding_popover_nesting_count() != 0
        {
            // Step 2.1. If throwExceptions is true, then throw a "InvalidStateError" DOMException.
            if throw_exceptions {
                return Err(Error::InvalidState(Some(
                    "Cannot show a popover while another popover is being shown or hidden.".into(),
                )));
            }

            // Step 2.2. Return.
            return Ok(());
        }

        // Step 3. Let validityResult be the result of running check popover validity given element,
        // false, and null. If this throws an exception, catch it, and set validityResult to that
        // exception.
        let validity_result = self.check_popover_validity(false, None);

        // Step 4. If validityResult is not true:
        if !matches!(validity_result, Ok(true)) {
            // Step 4.1. If throwExceptions is true and validityResult is a DOMException, then throw
            // validityResult.
            if throw_exceptions && let Err(error) = validity_result {
                return Err(error);
            }

            // Step 4.2. Return.
            return Ok(());
        }

        // Step 5. Assert: element's popover trigger is null.
        assert!(self.popover_trigger().is_none());

        // Step 6. Assert: element is not in document's top layer.
        assert!(!overlay_handler.is_in_the_top_layer(element));

        // Step 7. Set document's showing popover to true.
        overlay_handler.set_showing_popover(true);

        // Step 8. Let cleanupShowingSteps be the following step:
        let cleanup_showing_steps = || {
            // Step 8.1. Set document's showing popover to false.
            overlay_handler.set_showing_popover(false);
        };

        // Step 9. If the result of firing an event named beforetoggle, using ToggleEvent, with the
        // cancelable attribute initialized to true, the oldState attribute initialized to "closed",
        // the newState attribute initialized to "open", and the source attribute initialized to
        // source at element is false, then run cleanupShowingSteps and return.
        let event = ToggleEvent::new(
            cx,
            &self.owner_window(),
            atom!("beforetoggle"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            DOMString::from_static("closed"),
            DOMString::from_static("open"),
            source.map(|source| source.upcast::<Element>()),
        );
        if !event
            .upcast::<Event>()
            .fire(cx, self.upcast::<EventTarget>())
        {
            cleanup_showing_steps();
            return Ok(());
        }

        // Step 10. Set validityResult to the result of running check popover validity given
        // element, false, and document. If this throws an exception, catch it, and set
        // validityResult to that exception.
        let validity_result = self.check_popover_validity(false, Some(&document));

        // Step 11. If validityResult is not true:
        if !matches!(validity_result, Ok(true)) {
            // Step 11.1. Run cleanupShowingSteps.
            cleanup_showing_steps();

            // Step 11.2. If throwExceptions is true and validityResult is a DOMException, then
            // throw validityResult.
            if throw_exceptions && let Err(error) = validity_result {
                return Err(error);
            }

            // Step 11.3. Return.
            return Ok(());
        }

        // Step 12. Let shouldRestoreFocus be false.
        let mut should_restore_focus = false;

        // Step 13. Let originalType be the current state of element's popover attribute.
        let original_type = self.popover_state();

        // Step 14. Let effectiveType be originalType.
        let mut effective_type = original_type;

        let mut ancestor = None;

        // Step 15. If originalType is Auto or Hint, then:
        if matches!(original_type, PopoverState::Auto | PopoverState::Hint) {
            // Step 15.1. Let ancestor be the result of running topmost popover ancestor given
            // element, source, and true.
            ancestor = element.topmost_popover_ancestor(cx.no_gc(), source, true);

            // Step 15.2. If all of the following are true:
            // - ancestor is not null;
            // - ancestor's opened in popover mode is "hint"; and
            // - effectiveType is the Auto state,
            // then set effectiveType to the Hint state.
            if ancestor.as_ref().is_some_and(|ancestor| {
                ancestor.opened_in_popover_mode() == Some(PopoverState::Hint)
            }) && effective_type == PopoverState::Auto
            {
                effective_type = PopoverState::Hint;
            }

            // Step 15.3. Run hide popover stack until given document, ancestor, Hint,
            // shouldRestoreFocus, and true.
            overlay_handler.hide_popover_stack_until(
                cx,
                ancestor.as_deref(),
                PopoverState::Hint,
                should_restore_focus,
                true,
            );

            // Step 15.4. If effectiveType is Auto, then run hide popover stack until given
            // document, ancestor, Auto, shouldRestoreFocus, and true.
            if effective_type == PopoverState::Auto {
                overlay_handler.hide_popover_stack_until(
                    cx,
                    ancestor.as_deref(),
                    PopoverState::Auto,
                    should_restore_focus,
                    true,
                );
            }

            // Step 15.5. If originalType is not equal to the value of element's popover attribute:
            if original_type != self.popover_state() {
                // Step 15.5.1. Run cleanupShowingSteps.
                cleanup_showing_steps();

                // Step 15.5.2. If throwExceptions is true, then throw an "InvalidStateError" DOMException.
                if throw_exceptions {
                    return Err(Error::InvalidState(Some(
                        "The popover attribute changed while showing the popover.".into(),
                    )));
                }

                // Step 15.5.3. Return.
                return Ok(());
            }

            // Step 15.6. Set validityResult to the result of running check popover validity given
            // element, false, and document. If this throws an exception, catch it, and set
            // validityResult to that exception.
            let validity_result = self.check_popover_validity(false, Some(&document));

            // Step 15.7. If validityResult is not true:
            if !matches!(validity_result, Ok(true)) {
                // Step 15.7.1. Run cleanupShowingSteps.
                cleanup_showing_steps();

                // Step 15.7.2. If throwExceptions is true and validityResult is a DOMException,
                // then throw validityResult.
                if throw_exceptions && let Err(error) = validity_result {
                    return Err(error);
                }

                // Step 15.7.3. Return.
                return Ok(());
            }

            // Step 15.8. If the result of running topmost auto or hint popover on document is null,
            // then set shouldRestoreFocus to true.
            if overlay_handler
                .find_the_topmost_auto_or_hint_popover(cx.no_gc())
                .is_none()
            {
                should_restore_focus = true;
            }

            // Step 15.9. If effectiveType is Auto:
            if effective_type == PopoverState::Auto {
                // Step 15.9.1. Assert: document's showing auto popover list does not contain element.
                debug_assert!(!overlay_handler.showing_popover_list_contains(
                    cx.no_gc(),
                    PopoverState::Auto,
                    self
                ));

                // Step 15.9.2. Set element's opened in popover mode to "auto".
                self.set_opened_in_popover_mode(cx.no_gc(), Some(PopoverState::Auto));
            }
            // Otherwise:
            else {
                // Step 15.9.1. Assert: effectiveType is Hint.
                debug_assert!(effective_type == PopoverState::Hint);

                // Step 15.9.2. Assert: document's showing hint popover list does not contain element.
                debug_assert!(!overlay_handler.showing_popover_list_contains(
                    cx.no_gc(),
                    PopoverState::Hint,
                    self
                ));

                // Step 15.9.3. Set element's opened in popover mode to "hint".
                self.set_opened_in_popover_mode(cx.no_gc(), Some(PopoverState::Hint));
            }

            // Step 15.10. Set element's popover close watcher to the result of establishing a close
            // watcher given element's relevant global object, with:
            // - cancelAction being to return true.
            // - closeAction being to hide a popover given element, true, true, false, and null.
            // - getEnabledState being to return true.
            let close_watcher = InternalCloseWatcher::establish(
                cx.no_gc(),
                document.window(),
                &InternalCloseWatcherOwner::Popover(Dom::from_ref(self)),
            );
            self.set_popover_close_watcher(cx.no_gc(), close_watcher);
        }

        // Step 16. Set element's previously focused element to null.
        self.set_previously_focused_element(None, cx.no_gc());

        // Step 17. Let originallyFocusedElement be document's focused area of the document's DOM
        // anchor.
        let originally_focused_element = document
            .focus_handler()
            .focused_area()
            .element()
            .map(DomRoot::from_ref);

        // Step 18. Add an element to the top layer given element.
        overlay_handler.add_an_element_to_the_top_layer(element, cx.no_gc());

        // Step 19. If effectiveType is Hint and ancestor's opened in popover mode is "auto", then
        // set document's hint stack parent to ancestor.
        if effective_type == PopoverState::Hint &&
            let Some(ancestor) = &ancestor &&
            ancestor.opened_in_popover_mode() == Some(PopoverState::Auto)
        {
            overlay_handler.set_hint_stack_parent(Some(ancestor));
        }

        // Step 20. Set element's popover visibility state to showing.
        element.set_popover_visibility_state_showing(cx.no_gc(), true);

        // Step 21. Set element's popover trigger to source.
        self.set_popover_trigger(cx.no_gc(), source);

        // TODO: Step 22. Set element's implicit anchor element to source.

        // Step 23. Run the popover focusing steps given element.
        self.run_popover_focusing_steps(cx);

        // Step 24. If shouldRestoreFocus is true and element's popover attribute is not in the No
        // Popover state, then set element's previously focused element to
        // originallyFocusedElement.
        if should_restore_focus && self.popover_state() != PopoverState::NoPopover {
            self.set_previously_focused_element(originally_focused_element.as_deref(), cx.no_gc());
        }

        // Step 25. Run cleanupShowingSteps.
        cleanup_showing_steps();

        // Step 26. Queue a popover toggle event task given element, "closed", "open", and source.
        self.queue_a_popover_toggle_event_task("closed", "open", source);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#queue-a-popover-toggle-event-task>
    fn queue_a_popover_toggle_event_task(
        &self,
        old_state: &'static str,
        new_state: &'static str,
        source: Option<&HTMLElement>,
    ) {
        // TODO: Step 1. If element's popover toggle task tracker is not null, then:
        // TODO: Step 1.1. Set oldState to element's popover toggle task tracker's old state.
        // TODO: Step 1.2. Remove element's popover toggle task tracker's task from its task queue.
        // TODO: Step 1.3. Set element's popover toggle task tracker to null.

        // Step 2. Queue an element task given the DOM manipulation task source and element to run
        // the following steps:
        let this = Trusted::new(self);
        let trusted_source = source.map(|source| Trusted::new(source.upcast::<Element>()));
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(fire_popover_toggle_event: move |cx| {
                let this = this.root();
                let source = trusted_source.map(|source| source.root());

                // Step 2.1. Fire an event named toggle at element, using ToggleEvent, with the
                // oldState attribute initialized to oldState, the newState attribute initialized to
                // newState, and the source attribute initialized to source.
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
                event.upcast::<Event>().fire(cx, this.upcast::<EventTarget>());

                // TODO: Step 2.2. Set element's popover toggle task tracker to null.
            }));

        // TODO: Step 3. Set element's popover toggle task tracker to a struct with task set to the
        // just-queued task and old state set to oldState.
    }

    /// <https://html.spec.whatwg.org/multipage/#hide-popover-algorithm>
    pub(crate) fn hide_popover(
        &self,
        cx: &mut JSContext,
        focus_previous_element: bool,
        mut fire_events: bool,
        throw_exceptions: bool,
        source: Option<&HTMLElement>,
    ) -> Fallible<()> {
        let element = self.upcast::<Element>();

        // Step 1. Let validityResult be the result of running check popover validity given element,
        // true, and null. If this throws an exception, catch it, and let validityResult be that
        // exception.
        let validity_result = self.check_popover_validity(true, None);

        // Step 2. If validityResult is not true:
        if !matches!(validity_result, Ok(true)) {
            // Step 2.1. If throwExceptions is true and validityResult is a DOMException, then throw
            // validityResult.
            if throw_exceptions && let Err(error) = validity_result {
                return Err(error);
            }

            // Step 2.2. Return.
            return Ok(());
        }

        // Step 3. Let document be element's node document.
        let document = self.owner_document();
        let overlay_handler = document.overlay_handler();

        // Step 4. Let nestedHide be element's popover hiding.
        let nested_hide = self.popover_hiding();

        // Step 5. Set element's popover hiding to true.
        self.set_popover_hiding(cx.no_gc(), true);

        // Step 6. If nestedHide is true, then set fireEvents to false.
        if nested_hide {
            fire_events = false;
        }

        // Step 7. Increment document's hiding popover nesting count.
        overlay_handler.increment_hiding_popover_nesting_count();

        // Step 8. Let cleanupSteps be the following steps:
        let cleanup_steps = |no_gc: &NoGC| {
            // Step 8.1. If nestedHide is false, then set element's popover hiding to false.
            if !nested_hide {
                self.set_popover_hiding(no_gc, false);
            }

            // Step 8.2. Decrement document's hiding popover nesting count.
            overlay_handler.decrement_hiding_popover_nesting_count();
        };

        // Step 9. Let autoPopoverListContainsElement be true if document's showing auto popover
        // list contains element; otherwise false.
        let auto_popover_list_contains_element =
            overlay_handler.showing_popover_list_contains(cx.no_gc(), PopoverState::Auto, self);

        // Step 10. Let hintPopoverListContainsElement be true if document's showing hint popover
        // list contains element; otherwise false.
        let hint_popover_list_contains_element =
            overlay_handler.showing_popover_list_contains(cx.no_gc(), PopoverState::Hint, self);

        // Step 11. If element's opened in popover mode is "auto" or "hint", then:
        if matches!(
            self.opened_in_popover_mode(),
            Some(PopoverState::Auto) | Some(PopoverState::Hint)
        ) {
            // Step 11.1. If hintPopoverListContainsElement is true, then run hide popover stack
            // until given document, element, Hint, focusPreviousElement, and fireEvents.
            if hint_popover_list_contains_element {
                overlay_handler.hide_popover_stack_until(
                    cx,
                    Some(self),
                    PopoverState::Hint,
                    focus_previous_element,
                    fire_events,
                );
            }

            // Step 11.2. If element is document's hint stack parent, then run hide popover stack
            // until given document, null, Hint, focusPreviousElement, and fireEvents.
            if overlay_handler.is_hint_stack_parent(self) {
                overlay_handler.hide_popover_stack_until(
                    cx,
                    None,
                    PopoverState::Hint,
                    focus_previous_element,
                    fire_events,
                );
            }

            // Step 11.3. If autoPopoverListContainsElement is true, then run hide popover stack
            // until given document, element, Auto, focusPreviousElement, and fireEvents.
            if auto_popover_list_contains_element {
                overlay_handler.hide_popover_stack_until(
                    cx,
                    Some(self),
                    PopoverState::Auto,
                    focus_previous_element,
                    fire_events,
                );
            }

            // Step 11.4. Set validityResult to the result of running check popover validity given
            // element, true, and document. If this throws an exception, catch it, and set
            // validityResult to that exception.
            let validity_result = self.check_popover_validity(true, Some(&document));

            // Step 11.5. If validityResult is not true:
            if !matches!(validity_result, Ok(true)) {
                // Step 11.5.1. Run cleanupSteps.
                cleanup_steps(cx);

                // Step 11.5.2. If throwExceptions is true and validityResult is a DOMException,
                // then throw validityResult.
                if throw_exceptions && let Err(error) = validity_result {
                    return Err(error);
                }

                // Step 11.5.3. Return.
                return Ok(());
            }
        }

        // Step 12. If fireEvents is true:
        if fire_events {
            // Step 12.1. Fire an event named beforetoggle, using ToggleEvent, with the oldState
            // attribute initialized to "open", the newState attribute initialized to "closed", and
            // the source attribute set to source at element.
            let event = ToggleEvent::new(
                cx,
                &self.owner_window(),
                atom!("beforetoggle"),
                EventBubbles::DoesNotBubble,
                EventCancelable::NotCancelable,
                DOMString::from_static("open"),
                DOMString::from_static("closed"),
                source.map(|source| source.upcast::<Element>()),
            );
            event
                .upcast::<Event>()
                .fire(cx, self.upcast::<EventTarget>());

            // Step 12.2. Set validityResult to the result of running check popover validity given
            // element, true, and document, catching any exceptions.
            let validity_result = self.check_popover_validity(true, Some(&document));

            // Step 12.2. If validityResult is not true:
            if !matches!(validity_result, Ok(true)) {
                // Step 12.2.1. Run cleanupSteps.
                cleanup_steps(cx);

                // Step 12.2.2. If throwExceptions is true and validityResult is a DOMException,
                // then throw validityResult.
                if throw_exceptions && let Err(error) = validity_result {
                    return Err(error);
                }

                // Step 12.2.3. Return.
                return Ok(());
            }

            // Step 12.4. Request an element to be removed from the top layer given element.
            overlay_handler
                .request_an_element_to_be_removed_from_the_top_layer(element, cx.no_gc());

            // TODO: Step 12.5. Set element's implicit anchor element to null.
        }
        // Step 13. Otherwise, remove an element from the top layer immediately given element.
        else {
            overlay_handler.remove_an_element_from_the_top_layer_immediately(element, cx.no_gc());
        }

        // Step 14. Set element's popover trigger to null.
        self.set_popover_trigger(cx.no_gc(), None);

        // Step 15. Set element's opened in popover mode to null.
        self.set_opened_in_popover_mode(cx.no_gc(), None);

        // Step 16. Set element's popover visibility state to hidden.
        element.set_popover_visibility_state_showing(cx.no_gc(), false);

        // Step 17. If element is document's hint stack parent, or document's showing hint popover
        // list is empty, then set document's hint stack parent to null.
        if overlay_handler.is_hint_stack_parent(self) ||
            overlay_handler
                .showing_popover_list_unrooted(cx.no_gc(), PopoverState::Hint)
                .is_empty()
        {
            overlay_handler.set_hint_stack_parent(None);
        }

        // Step 18. If fireEvents is true, then queue a popover toggle event task given element,
        // "open", "closed", and source.
        if fire_events {
            self.queue_a_popover_toggle_event_task("open", "closed", source);
        }

        // Step 19. Let previouslyFocusedElement be element's previously focused element.
        // Step 20. If previouslyFocusedElement is not null, then:
        if let Some(previously_focused_element) = self.previously_focused_element(cx.no_gc()) {
            // Step 20.1. Set element's previously focused element to null.
            self.set_previously_focused_element(None, cx.no_gc());

            // Step 20.2. If focusPreviousElement is true and document's focused area of the
            // document's DOM anchor is a shadow-including inclusive descendant of element, then
            // run the focusing steps for previouslyFocusedElement; the viewport should not be
            // scrolled by doing this step.
            if focus_previous_element &&
                document
                    .focus_handler()
                    .focused_area()
                    .dom_anchor(&document)
                    .inclusive_ancestors(ShadowIncluding::Yes)
                    .any(|node| &*node == self.upcast::<Node>())
            {
                previously_focused_element
                    .upcast::<Node>()
                    .run_the_focusing_steps(cx, None, FocusTrigger::Script);
            }
        }

        // Step 21. If element's popover close watcher is not null:
        // Step 21.1. Destroy element's popover close watcher.
        // Step 21.2. Set element's popover close watcher to null.
        if let Some(close_watcher) = self.take_popover_close_watcher(cx.no_gc()) {
            close_watcher.destroy(cx.no_gc());
        }

        // Step 22. Run cleanupSteps.
        cleanup_steps(cx.no_gc());
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#popover-focusing-steps>
    fn run_popover_focusing_steps(&self, cx: &mut JSContext) {
        // TODO: Step 1. If the allow focus steps given subject's node document return false, then
        // return.

        // Step 2. If subject is a dialog element, then run the dialog focusing steps given subject
        // and return.
        if let Some(dialog) = self.downcast::<HTMLDialogElement>() {
            dialog.run_dialog_focusing_steps(cx);
            return;
        }

        // Step 3. If subject has the autofocus attribute, then let control be subject.
        let node = self.upcast::<Node>();
        rooted!(&in(cx) let control = if self.Autofocus() {
            node.get_the_focusable_area(cx, FocusTrigger::Script)
        }
        // Step 4. Otherwise, let control be the autofocus delegate for subject given "other".
        else {
            node.autofocus_delegate(cx, FocusTrigger::Script)
        });

        // Step 5. If control is null, then return.
        let Some(control) = &*control else {
            return;
        };

        // Step 6. Run the focusing steps given control.
        // FIXME: Use the focusing steps once they support a focusable area as an argument.
        self.owner_document()
            .focus_handler()
            .focus(cx, control, FocusTrigger::Script);

        // TODO: Step 7. Let topDocument be the active document of control's node document's
        // node navigable's top-level traversable.
        // TODO: Step 8. If control's node document's origin is not the same as the origin of
        // topDocument, then return.
        // TODO: Step 9. Empty topDocument's autofocus candidates.
        // TODO: Step 10. Set topDocument's autofocus processed flag to true.
    }

    /// <https://html.spec.whatwg.org/multipage/#check-popover-validity>
    pub(crate) fn check_popover_validity(
        &self,
        expected_to_be_showing: bool,
        expected_document: Option<&Document>,
    ) -> Fallible<bool> {
        // Step 1. If element's popover attribute is in the No Popover state, then throw a
        // "NotSupportedError" DOMException.
        if self.popover_state() == NoPopover {
            return Err(Error::NotSupported(Some(
                "Element does not have a popover attribute.".into(),
            )));
        }

        // Step 2. If any of the following are true:
        // - expectedToBeShowing is true and element's popover visibility state is not showing; or
        // - expectedToBeShowing is false and element's popover visibility state is not hidden,
        // then return false.
        let element = self.upcast::<Element>();
        if expected_to_be_showing != element.popover_visibility_state_showing() {
            return Ok(false);
        }

        // Step 3. If any of the following are true:
        // - expectedToBeShowing is false and element is not connected;
        // - expectedToBeShowing is false and element's node document is not fully active;
        // - expectedDocument is not null and element's node document is not expectedDocument;
        // - element is a dialog element and its is modal is set to true; or
        // - element's fullscreen flag is set.
        // then throw an "InvalidStateError" DOMException.
        let document = self.owner_document();
        if (!expected_to_be_showing &&
            (!self.upcast::<Node>().is_connected() || !document.is_fully_active())) ||
            expected_document.is_some_and(|expected| *expected != *document) ||
            (self.is::<HTMLDialogElement>() && element.state().contains(ElementState::MODAL)) ||
            element.state().contains(ElementState::FULLSCREEN)
        {
            return Err(Error::InvalidState(Some(
                "Element is not in a valid state to show or hide a popover.".into(),
            )));
        }

        // Step 4. Return true.
        Ok(true)
    }

    /// <https://html.spec.whatwg.org/multipage/#popover-target-attribute-activation-behavior>
    pub(crate) fn popover_target_attribute_activation_behavior(
        &self,
        cx: &mut JSContext,
        event_target: &Node,
    ) {
        // 1. Let popover be node's popover target element.
        let popover = self.popover_target_element(cx);

        // 2. If popover is null, then return.
        let Some(popover) = popover else {
            return;
        };

        // 3. If eventTarget is a shadow-including inclusive descendant of popover and popover
        // is a shadow-including descendant of node, then return.
        let popover_node = popover.upcast::<Node>();
        if popover_node.is_shadow_including_inclusive_ancestor_of(event_target) &&
            self.upcast::<Node>()
                .is_shadow_including_inclusive_ancestor_of(popover_node)
        {
            return;
        }

        let popover_showing = popover
            .upcast::<Element>()
            .popover_visibility_state_showing();
        let popover_target_action = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("popovertargetaction"))
            .to_ascii_lowercase();

        // 4. If node's popovertargetaction attribute is in the show state and popover's popover
        // visibility state is showing, then return.
        if popover_target_action == "show" && popover_showing {
            return;
        }

        // 5. If node's popovertargetaction attribute is in the hide state and popover's popover
        // visibility state is hidden, then return.
        if popover_target_action == "hide" && !popover_showing {
            return;
        }

        // 6. If popover's popover visibility state is showing, then run the hide popover algorithm
        // given popover, true, true, false, and node.
        if popover_showing {
            popover
                .hide_popover(cx, true, true, false, Some(self))
                .expect("This shouldn't fail in this codepath");
        }
        // 7. Otherwise:
        else {
            // 7.1. Let validity be the result of running check popover validity given popover,
            // false, and null. If this throws an exception, catch it, and let validity be false.
            let validity = popover.check_popover_validity(false, None);

            // 7.2. If validity is true, then run show popover given popover, false, and node.
            if matches!(validity, Ok(true)) {
                popover
                    .show_popover(cx, false, Some(self))
                    .expect("This shouldn't fail in this codepath");
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#popover-target-element>
    pub(crate) fn popover_target_element(
        &self,
        cx: &mut JSContext,
    ) -> Option<DomRoot<HTMLElement>> {
        // 1. If node is not a button, then return null.
        if !self.is_a_button() {
            return None;
        }

        // 2. If node is disabled, then return null.
        if self.is_form_element_disabled(cx.no_gc()) {
            return None;
        }

        // 3. If node has a form owner and node is a submit button, then return null.
        if self
            .downcast::<HTMLButtonElement>()
            .is_some_and(|button| button.form_owner().is_some() && button.is_submit_button()) ||
            self.downcast::<HTMLInputElement>()
                .is_some_and(|input| input.form_owner().is_some() && input.is_submit_button())
        {
            return None;
        }

        // 4. Let popoverElement be the result of running node's get the popovertarget-associated element.
        let popover_target_value = self
            .upcast::<Element>()
            .get_attribute_string_value(&local_name!("popovertarget"))?
            .into();

        let root_node = self
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());

        let popover_element = if let Some(document) = root_node.downcast::<Document>() {
            document.GetElementById(cx, popover_target_value)
        } else if let Some(document_fragment) = root_node.downcast::<DocumentFragment>() {
            document_fragment.GetElementById(cx, popover_target_value)
        } else {
            unreachable!("Element must be in a document or document fragment")
        };

        // 5. If popoverElement is null, then return null.
        let popover_element = DomRoot::downcast::<HTMLElement>(popover_element?)?;

        // 6. If popoverElement's popover attribute is in the No Popover state, then return null.
        if popover_element.popover_state() == PopoverState::NoPopover {
            return None;
        }

        // 7. Return popoverElement.
        Some(popover_element)
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-popover-stack-position>
    pub(crate) fn popover_stack_position(&self, cx: &mut JSContext) -> usize {
        let document = self.upcast::<Element>().owner_document();

        // Step 1. Let hintList be popover's node document's showing hint popover list.
        let hint_list = document.overlay_handler().showing_popover_list_unrooted(cx.no_gc(), PopoverState::Hint);

        // Step 2. Let autoList be popover's node document's showing auto popover list.
        let auto_list = document.overlay_handler().showing_popover_list_unrooted(cx.no_gc(), PopoverState::Auto);

        // Step 3. If popover is in hintList, then return the index of popover in hintList + the
        // size of autoList + 1.
        if let Some(hint_list_index) = hint_list.iter().position(|popover| popover == self) {
            return hint_list_index + auto_list.len() + 1;
        }

        // Step 4. If popover is in autoList, then return the index of popover in autoList + 1.
        if let Some(auto_list_index) = auto_list.iter().position(|popover| popover == self) {
            return auto_list_index + 1;
        }


        // Step 5. Return 0.
        0
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-button>
    fn is_a_button(&self) -> bool {
        self.is::<HTMLButtonElement>() ||
            self.downcast::<HTMLInputElement>().is_some_and(|input| {
                matches!(
                    *input.input_type(),
                    InputType::Submit(_) |
                        InputType::Reset(_) |
                        InputType::Image(_) |
                        InputType::Button(_)
                )
            })
    }

    pub(crate) fn opened_in_popover_mode(&self) -> Option<PopoverState> {
        self.upcast::<Element>()
            .rare_data()
            .as_ref()?
            .opened_in_popover_mode
    }

    fn set_opened_in_popover_mode(&self, no_gc: &NoGC, mode: Option<PopoverState>) {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .opened_in_popover_mode = mode;
    }

    fn popover_trigger(&self) -> Option<DomRoot<HTMLElement>> {
        self.upcast::<Element>()
            .rare_data()
            .as_ref()?
            .popover_trigger
            .get()
    }

    fn set_popover_trigger(&self, no_gc: &NoGC, trigger: Option<&HTMLElement>) {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .popover_trigger
            .set(trigger);
    }

    fn popover_hiding(&self) -> bool {
        self.upcast::<Element>()
            .rare_data()
            .as_ref()
            .is_some_and(|rare_data| rare_data.popover_hiding)
    }

    fn set_popover_hiding(&self, no_gc: &NoGC, value: bool) {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .popover_hiding = value;
    }

    fn set_popover_close_watcher(&self, no_gc: &NoGC, close_watcher: Rc<InternalCloseWatcher>) {
        self.upcast::<Element>()
            .ensure_rare_data(no_gc)
            .popover_close_watcher
            .safe_borrow_mut(no_gc)
            .replace(close_watcher);
    }

    fn take_popover_close_watcher(&self, no_gc: &NoGC) -> Option<Rc<InternalCloseWatcher>> {
        self.upcast::<Element>()
            .rare_data()
            .as_ref()?
            .popover_close_watcher
            .safe_borrow_mut(no_gc)
            .take()
    }
}

impl HTMLElementMethods<crate::DomTypeHolder> for HTMLElement {
    /// <https://html.spec.whatwg.org/multipage/#the-style-attribute>
    fn Style(&self, cx: &mut JSContext) -> DomRoot<CSSStyleDeclaration> {
        self.style_decl.or_init(|| {
            let global = self.owner_window();
            CSSStyleDeclaration::new(
                cx,
                &global,
                CSSStyleOwner::Element(Dom::from_ref(self.upcast())),
                None,
                CSSModificationAccess::ReadWrite,
            )
        })
    }

    // https://html.spec.whatwg.org/multipage/#attr-title
    make_getter!(Title, "title");
    // https://html.spec.whatwg.org/multipage/#attr-title
    make_setter!(SetTitle, "title");

    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_getter!(Lang, "lang");
    // https://html.spec.whatwg.org/multipage/#attr-lang
    make_setter!(SetLang, "lang");

    // https://html.spec.whatwg.org/multipage/#the-dir-attribute
    make_enumerated_getter!(
        Dir,
        "dir",
        "ltr" | "rtl" | "auto",
        missing => "",
        invalid => ""
    );

    // https://html.spec.whatwg.org/multipage/#the-dir-attribute
    make_setter!(SetDir, "dir");

    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_getter!(Hidden, "hidden");
    // https://html.spec.whatwg.org/multipage/#dom-hidden
    make_bool_setter!(SetHidden, "hidden");

    // https://html.spec.whatwg.org/multipage/#the-inert-attribute
    make_bool_getter!(Inert, "inert");
    make_bool_setter!(SetInert, "inert");

    // https://html.spec.whatwg.org/multipage/#globaleventhandlers
    global_event_handlers!(NoOnload);

    /// <https://html.spec.whatwg.org/multipage/#dom-dataset>
    fn Dataset(&self, cx: &mut JSContext) -> DomRoot<DOMStringMap> {
        self.dataset
            .or_init(|| DOMStringMap::new(cx, self.upcast()))
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onerror>
    fn GetOnerror(&self, cx: &mut JSContext) -> Option<RootedCallback<OnErrorEventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnerror(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "error")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onerror>
    fn SetOnerror(
        &self,
        cx: &mut JSContext,
        listener: Option<RootedCallback<OnErrorEventHandlerNonNull>>,
    ) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnerror(cx, listener)
            }
        } else {
            // special setter for error
            self.upcast::<EventTarget>()
                .set_error_event_handler(cx, "error", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onload>
    fn GetOnload(&self, cx: &mut JSContext) -> Option<RootedCallback<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnload(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "load")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onload>
    fn SetOnload(&self, cx: &mut JSContext, listener: Option<RootedCallback<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnload(cx, listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common(cx, "load", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onblur>
    fn GetOnblur(&self, cx: &mut JSContext) -> Option<RootedCallback<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnblur(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "blur")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onblur>
    fn SetOnblur(&self, cx: &mut JSContext, listener: Option<RootedCallback<EventHandlerNonNull>>) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnblur(cx, listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common(cx, "blur", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onfocus>
    fn GetOnfocus(&self, cx: &mut JSContext) -> Option<RootedCallback<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnfocus(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "focus")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onfocus>
    fn SetOnfocus(
        &self,
        cx: &mut JSContext,
        listener: Option<RootedCallback<EventHandlerNonNull>>,
    ) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnfocus(cx, listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common(cx, "focus", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onresize>
    fn GetOnresize(&self, cx: &mut JSContext) -> Option<RootedCallback<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnresize(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "resize")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onresize>
    fn SetOnresize(
        &self,
        cx: &mut JSContext,
        listener: Option<RootedCallback<EventHandlerNonNull>>,
    ) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnresize(cx, listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common(cx, "resize", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onscroll>
    fn GetOnscroll(&self, cx: &mut JSContext) -> Option<RootedCallback<EventHandlerNonNull>> {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().GetOnscroll(cx)
            } else {
                None
            }
        } else {
            self.upcast::<EventTarget>()
                .get_event_handler_common(cx, "scroll")
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#handler-onscroll>
    fn SetOnscroll(
        &self,
        cx: &mut JSContext,
        listener: Option<RootedCallback<EventHandlerNonNull>>,
    ) {
        if self.is_body_or_frameset() {
            let document = self.owner_document();
            if document.has_browsing_context() {
                document.window().SetOnscroll(cx, listener)
            }
        } else {
            self.upcast::<EventTarget>()
                .set_event_handler_common(cx, "scroll", listener)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#attr-itemtype>
    fn Itemtypes(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemtype"));

        if atoms.is_empty() {
            return None;
        }

        Some(
            FxHashSet::from_iter(
                atoms
                    .iter()
                    .map(|attr_value| DOMString::from(String::from(attr_value.trim()))),
            )
            .into_iter()
            .collect(),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#names:-the-itemprop-attribute>
    fn PropertyNames(&self) -> Option<Vec<DOMString>> {
        let atoms = self
            .element
            .get_tokenlist_attribute(&local_name!("itemprop"));

        if atoms.is_empty() {
            return None;
        }

        Some(
            FxHashSet::from_iter(
                atoms
                    .iter()
                    .map(|attr_value| DOMString::from(String::from(attr_value.trim()))),
            )
            .into_iter()
            .collect(),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-click>
    fn Click(&self, cx: &mut JSContext) {
        // 1. If this element is a form control that is disabled, then return.
        if self.is_form_element_disabled(cx.no_gc()) {
            return;
        }

        // 2. If this element's click in progress flag is set, then return.
        let element = self.as_element();
        if element.click_in_progress() {
            return;
        }

        // 3. Set this element's click in progress flag.
        element.set_click_in_progress(true);

        // 4. Fire a synthetic pointer event named click at this element, with the not trusted flag set.
        self.upcast::<Node>()
            .fire_synthetic_pointer_event_not_trusted(cx, atom!("click"));

        // 5. Unset this element's click in progress flag.
        element.set_click_in_progress(false);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-focus>
    fn Focus(&self, cx: &mut JSContext, options: &FocusOptions) {
        // 1. If the allow focus steps given this's node document return false, then return.
        // TODO: Implement this.

        // 2. Run the focusing steps for this.
        if !self
            .upcast::<Node>()
            .run_the_focusing_steps(cx, None, FocusTrigger::Script)
        {
            // The specification seems to imply we should scroll into view even if this element
            // is not a focusable area. No browser does this, so we return early in that case.
            // See https://github.com/whatwg/html/issues/12231.
            return;
        }

        // > 3. If options["focusVisible"] is true, or does not exist but in an
        // >    implementation-defined  way the user agent determines it would be best to do so,
        // >    then indicate focus. TODO: Implement this.

        // > 4. If options["preventScroll"] is false, then scroll a target into view given this,
        // >    "auto", "center", and "center".
        if !options.preventScroll {
            let scroll_axis = ScrollAxisState {
                position: ScrollLogicalPosition::Center,
                requirement: ScrollRequirement::IfNotVisible,
            };
            self.upcast::<Element>().scroll_into_view_with_options(
                cx,
                ScrollBehavior::Smooth,
                scroll_axis,
                scroll_axis,
                None,
                None,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-blur>
    fn Blur(&self, cx: &mut JSContext) {
        // TODO: Run the unfocusing steps. Focus the top-level document, not
        //       the current document.
        if !self.as_element().focus_state() {
            return;
        }
        // <https://html.spec.whatwg.org/multipage/#unfocusing-steps>
        self.owner_document().focus_handler().focus(
            cx,
            &FocusableArea::Viewport,
            FocusTrigger::Script,
        );
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-scrollparent>
    #[expect(unsafe_code)]
    fn ScrollParent(&self, cx: &mut JSContext) -> Option<DomRoot<Element>> {
        self.owner_window()
            .scroll_container_query(
                Some(self.upcast()),
                ScrollContainerQueryFlags::ForScrollParent,
            )
            .and_then(|response| match response {
                ScrollContainerResponse::Viewport(..) => {
                    self.owner_document().GetScrollingElement(cx)
                },
                ScrollContainerResponse::Element(parent_node_address, ..) => {
                    let node = unsafe { from_untrusted_node_address(parent_node_address) };
                    DomRoot::downcast(node)
                },
            })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetparent>
    fn GetOffsetParent(&self, cx: &mut JSContext) -> Option<DomRoot<Element>> {
        if self.is::<HTMLBodyElement>() || self.element.is_root() {
            return None;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (element, _) = window.offset_parent_query(cx, node);

        element
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsettop>
    fn OffsetTop(&self, cx: &mut JSContext) -> i32 {
        if self.is_body_element() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(cx, node);

        rect.origin.y.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetleft>
    fn OffsetLeft(&self, cx: &mut JSContext) -> i32 {
        if self.is_body_element() {
            return 0;
        }

        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(cx, node);

        rect.origin.x.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetwidth>
    fn OffsetWidth(&self, cx: &mut JSContext) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(cx, node);

        rect.size.width.to_nearest_px()
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-htmlelement-offsetheight>
    fn OffsetHeight(&self, cx: &mut JSContext) -> i32 {
        let node = self.upcast::<Node>();
        let window = self.owner_window();
        let (_, rect) = window.offset_parent_query(cx, node);

        rect.size.height.to_nearest_px()
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute>
    fn InnerText(&self, cx: &mut JSContext) -> DOMString {
        self.get_inner_outer_text(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#set-the-inner-text-steps>
    fn SetInnerText(&self, cx: &mut JSContext, input: DOMString) {
        self.set_inner_text(cx, input)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-outertext>
    fn GetOuterText(&self, cx: &mut JSContext) -> Fallible<DOMString> {
        Ok(self.get_inner_outer_text(cx))
    }

    /// <https://html.spec.whatwg.org/multipage/#the-innertext-idl-attribute:dom-outertext-2>
    fn SetOuterText(&self, cx: &mut JSContext, input: DOMString) -> Fallible<()> {
        // Step 1: If this's parent is null, then throw a "NoModificationAllowedError" DOMException.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return Err(Error::NoModificationAllowed(Some(
                "Cannot modify HTML element as its parent element is null".into(),
            )));
        };

        let node = self.upcast::<Node>();
        let document = self.owner_document();

        // Step 2: Let next be this's next sibling.
        let next = node.GetNextSibling();

        // Step 3: Let previous be this's previous sibling.
        let previous = node.GetPreviousSibling();

        // Step 4: Let fragment be the rendered text fragment for the given value given this's node
        // document.
        let fragment = self.rendered_text_fragment(cx, input);

        // Step 5: If fragment has no children, then append a new Text node whose data is the empty
        // string and node document is this's node document to fragment.
        if fragment.upcast::<Node>().children_count() == 0 {
            let text_node = Text::new(cx, DOMString::new(), &document);

            fragment
                .upcast::<Node>()
                .AppendChild(cx, text_node.upcast())?;
        }

        // Step 6: Replace this with fragment within this's parent.
        parent.ReplaceChild(cx, fragment.upcast(), node)?;

        // Step 7: If next is non-null and next's previous sibling is a Text node, then merge with
        // the next text node given next's previous sibling.
        if let Some(next_sibling) = next &&
            let Some(node) = next_sibling.GetPreviousSibling()
        {
            Self::merge_with_the_next_text_node(cx, &node);
        }

        // Step 8: If previous is a Text node, then merge with the next text node given previous.
        if let Some(previous) = previous {
            Self::merge_with_the_next_text_node(cx, &previous)
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-translate>
    fn Translate(&self) -> bool {
        self.as_element().is_translate_enabled()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-translate>
    fn SetTranslate(&self, cx: &mut JSContext, yesno: bool) {
        self.as_element().set_string_attribute(
            cx,
            &html5ever::local_name!("translate"),
            match yesno {
                true => DOMString::from_static("yes"),
                false => DOMString::from_static("no"),
            },
        );
    }

    // https://html.spec.whatwg.org/multipage/#dom-contenteditable
    make_enumerated_getter!(
        ContentEditable,
        "contenteditable",
        "true" | "false" | "plaintext-only",
        missing => "inherit",
        invalid => "inherit",
        empty => "true"
    );

    /// <https://html.spec.whatwg.org/multipage/#dom-contenteditable>
    fn SetContentEditable(&self, cx: &mut JSContext, value: DOMString) -> ErrorResult {
        let attr_name = &local_name!("contenteditable");
        if value.eq_ignore_ascii_case("inherit") {
            // > On setting, if the new value is an ASCII case-insensitive match for the string "inherit", then the content attribute must be removed,
            self.element.remove_attribute_by_name(cx, attr_name);
        } else if value.eq_ignore_ascii_case("true") ||
            value.eq_ignore_ascii_case("false") ||
            value.eq_ignore_ascii_case("plaintext-only")
        {
            // > if the new value is an ASCII case-insensitive match for the string "true", then the content attribute must be set to the string "true",
            // > if the new value is an ASCII case-insensitive match for the string "plaintext-only", then the content attribute must be set to the string "plaintext-only",
            // > if the new value is an ASCII case-insensitive match for the string "false", then the content attribute must be set to the string "false",
            let lower_value = value.to_ascii_lowercase();
            self.element
                .set_attribute(cx, attr_name, AttrValue::String(lower_value));
        } else {
            // > and otherwise the attribute setter must throw a "SyntaxError" DOMException.
            return Err(Error::Syntax(Some(
                "Invalid attribute for HTML element".into(),
            )));
        };
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-iscontenteditable>
    fn IsContentEditable(&self) -> bool {
        // > The isContentEditable IDL attribute, on getting, must return true if the element is either an editing host or editable, and false otherwise.
        self.upcast::<Node>().is_editable_or_editing_host()
    }

    /// <https://html.spec.whatwg.org/multipage#dom-attachinternals>
    fn AttachInternals(&self, cx: &mut JSContext) -> Fallible<DomRoot<ElementInternals>> {
        // Step 1: If this's is value is not null, then throw a "NotSupportedError" DOMException
        if self.element.get_is().is_some() {
            return Err(Error::NotSupported(Some(
                "Local name of HTML element must not be set".into(),
            )));
        }

        // Step 2: Let definition be the result of looking up a custom element definition
        let lookup_registry = {
            // TODO: Remove this fallback when Node::adopt is aligned according to specs.
            //       Currently elements carry stale global registry from another document.
            let registry = self.as_element().custom_element_registry();
            if registry
                .as_ref()
                .is_some_and(|registry| registry.is_scoped())
            {
                registry
            } else {
                self.upcast::<Node>().owner_doc().custom_element_registry()
            }
        };
        let definition = CustomElementRegistry::lookup_custom_element_definition(
            lookup_registry.as_deref(),
            self.upcast::<Element>().namespace(),
            self.as_element().local_name(),
            None,
        );

        // Step 3: If definition is null, then throw an "NotSupportedError" DOMException
        let definition = match definition {
            Some(definition) => definition,
            None => {
                return Err(Error::NotSupported(Some(
                    "Custom element definition is missing".into(),
                )));
            },
        };

        // Step 4: If definition's disable internals is true, then throw a "NotSupportedError" DOMException
        if definition.disable_internals {
            return Err(Error::NotSupported(Some(
                "Custom element definition's `disabledFeatures` must not include \"internals\""
                    .into(),
            )));
        }

        // Step 5: If this's attached internals is non-null, then throw an "NotSupportedError" DOMException
        let internals = self.ensure_element_internals(cx);
        if internals.attached() {
            return Err(Error::NotSupported(Some(
                "HTML element's internals are already attached".into(),
            )));
        }

        // Step 6: If this's custom element state is not "precustomized" or "custom",
        // then throw a "NotSupportedError" DOMException.
        if !matches!(
            self.element.get_custom_element_state(),
            CustomElementState::Precustomized | CustomElementState::Custom
        ) {
            return Err(Error::NotSupported(Some(
                "HTML element is not yet upgraded".into(),
            )));
        }

        if self.is_form_associated_custom_element() {
            self.element.init_state_for_internals(cx.no_gc());
        }

        // Step 6-7: Set this's attached internals to a new ElementInternals instance
        internals.set_attached();
        Ok(internals)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-showpopover>
    fn ShowPopover(&self, cx: &mut JSContext, options: &ShowPopoverOptions) -> Fallible<()> {
        // Step 1. Let source be options["source"] if it exists; otherwise, null.
        // Step 2. Run show popover given this, true, and source.
        self.show_popover(cx, true, options.source.as_deref())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-hidepopover>
    fn HidePopover(&self, cx: &mut JSContext) -> Fallible<()> {
        // 1. Run the hide popover algorithm given this, true, true, true, and null.
        self.hide_popover(cx, true, true, true, None)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-togglepopover>
    fn TogglePopover(
        &self,
        cx: &mut JSContext,
        options: TogglePopoverOptionsOrBoolean,
    ) -> Fallible<bool> {
        // Step 1. Let force be null.
        let (force, source) = match &options {
            // Step 2. If options is a boolean, set force to options.
            TogglePopoverOptionsOrBoolean::Boolean(options) => (Some(*options), None),
            // Step 3. Otherwise, if options["force"] exists, set force to options["force"].
            // Step 4. Let source be options["source"] if it exists; otherwise, null.
            TogglePopoverOptionsOrBoolean::TogglePopoverOptions(options) => {
                (options.force, options.parent.source.as_deref())
            },
        };

        // Step 5. If this's popover visibility state is showing, and force is null or false, then
        // run the hide popover algorithm given this, true, true, true, and null.
        if self.upcast::<Element>().popover_visibility_state_showing() &&
            force.is_none_or(|force| !force)
        {
            self.hide_popover(cx, true, true, true, None)?
        }
        // Step 6. Otherwise, if force is null or true, then run show popover given this, true, and source.
        else if force.is_none_or(|force| force) {
            self.show_popover(cx, true, source)?;
        }
        // Step 7. Otherwise:
        else {
            // Step 7.1. Let expectedToBeShowing be true if this's popover visibility state is
            // showing; otherwise false.
            let expected_to_be_showing =
                self.upcast::<Element>().popover_visibility_state_showing();

            // Step 7.2. Run check popover validity given expectedToBeShowing, true, and null.
            self.check_popover_validity(expected_to_be_showing, None)?;
        }

        // Step 8. Return true if this's popover visibility state is showing; otherwise false.
        Ok(self.upcast::<Element>().popover_visibility_state_showing())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-popover>
    fn GetPopover(&self) -> Option<DOMString> {
        self.popover_state().reflect_attribute()
    }

    // https://html.spec.whatwg.org/multipage/#dom-popover
    fn SetPopover(&self, cx: &mut JSContext, value: Option<DOMString>) {
        let element = self.upcast::<Element>();
        if let Some(value) = value {
            element.set_string_attribute(cx, &local_name!("popover"), value);
        } else {
            element.remove_attribute_by_name(cx, &local_name!("popover"));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn Nonce(&self) -> DOMString {
        self.as_element().nonce_value().into()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-noncedelement-nonce>
    fn SetNonce(&self, cx: &mut JSContext, value: DOMString) {
        self.as_element()
            .update_nonce_internal_slot(String::from(value), cx.no_gc())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn Autofocus(&self) -> bool {
        self.element.has_attribute(&local_name!("autofocus"))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fe-autofocus>
    fn SetAutofocus(&self, cx: &mut JSContext, autofocus: bool) {
        self.element
            .set_bool_attribute(cx, &local_name!("autofocus"), autofocus);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn TabIndex(&self) -> i32 {
        self.element.tab_index()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-tabindex>
    fn SetTabIndex(&self, cx: &mut JSContext, tab_index: i32) {
        self.element
            .set_attribute(cx, &local_name!("tabindex"), tab_index.into());
    }

    // https://html.spec.whatwg.org/multipage/#dom-accesskey
    make_getter!(AccessKey, "accesskey");

    // https://html.spec.whatwg.org/multipage/#dom-accesskey
    make_setter!(SetAccessKey, "accesskey");

    /// <https://html.spec.whatwg.org/multipage/#dom-accesskeylabel>
    fn AccessKeyLabel(&self) -> DOMString {
        // The accessKeyLabel IDL attribute must return a string that represents the element's
        // assigned access key, if any. If the element does not have one, then the IDL attribute
        // must return the empty string.
        if !self.element.has_attribute(&local_name!("accesskey")) {
            return Default::default();
        }

        let access_key_string =
            String::from(self.element.get_string_attribute(&local_name!("accesskey")));

        #[cfg(target_os = "macos")]
        let access_key_label = format!("⌃⌥{access_key_string}");
        #[cfg(not(target_os = "macos"))]
        let access_key_label = format!("Alt+Shift+{access_key_string}");

        access_key_label.into()
    }
}

fn append_text_node_to_fragment(
    cx: &mut JSContext,
    document: &Document,
    fragment: &DocumentFragment,
    text: String,
) {
    let text = Text::new(cx, DOMString::from(text), document);
    fragment
        .upcast::<Node>()
        .AppendChild(cx, text.upcast())
        .unwrap();
}

impl HTMLElement {
    /// <https://html.spec.whatwg.org/multipage/#category-label>
    pub(crate) fn is_labelable_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLInputElement => !matches!(
                    *self.downcast::<HTMLInputElement>().unwrap().input_type(),
                    InputType::Hidden(_)
                ),
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLMeterElement |
                HTMLElementTypeId::HTMLOutputElement |
                HTMLElementTypeId::HTMLProgressElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#form-associated-custom-element>
    pub(crate) fn is_form_associated_custom_element(&self) -> bool {
        if let Some(definition) = self.as_element().get_custom_element_definition() {
            definition.is_autonomous() && definition.form_associated
        } else {
            false
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#category-listed>
    pub(crate) fn is_listed_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLFieldSetElement |
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLObjectElement |
                HTMLElementTypeId::HTMLOutputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-body-element-2>
    pub(crate) fn is_body_element(&self) -> bool {
        let self_node = self.upcast::<Node>();
        self_node.GetParentNode().is_some_and(|parent| {
            let parent_node = parent.upcast::<Node>();
            (self_node.is::<HTMLBodyElement>() || self_node.is::<HTMLFrameSetElement>()) &&
                parent_node.is::<HTMLHtmlElement>() &&
                self_node
                    .preceding_siblings()
                    .all(|n| !n.is::<HTMLBodyElement>() && !n.is::<HTMLFrameSetElement>())
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#category-submit>
    pub(crate) fn is_submittable_element(&self) -> bool {
        match self.upcast::<Node>().type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_id)) => match type_id {
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement => true,
                _ => self.is_form_associated_custom_element(),
            },
            _ => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    // This gets the nth label in tree order.
    pub(crate) fn label_at<'a>(
        &self,
        no_gc: &'a NoGC,
        index: u32,
    ) -> Option<UnrootedDom<'a, Node>> {
        let element = self.as_element();

        // Traverse entire tree for <label> elements that have
        // this as their control.
        // There is room for performance optimization, as we don't need
        // the actual result of GetControl, only whether the result
        // would match self.
        // (Even more room for performance optimization: do what
        // nodelist ChildrenList does and keep a mutation-aware cursor
        // around; this may be hard since labels need to keep working
        // even as they get detached into a subtree and reattached to
        // a document.)
        let root_element = element.root_element();
        let root_node = root_element.upcast::<Node>();
        root_node
            .traverse_preorder_unrooted(no_gc, ShadowIncluding::No)
            .filter_map(UnrootedDom::downcast::<HTMLLabelElement>)
            .filter(|elem| match elem.GetControl(no_gc) {
                Some(control) => &*control == self,
                _ => false,
            })
            .nth(index as usize)
            .map(UnrootedDom::upcast)
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    // This counts the labels of the element, to support NodeList::Length
    pub(crate) fn labels_count(&self, no_gc: &NoGC) -> u32 {
        // see label_at comments about performance
        let element = self.as_element();
        let root_element = element.root_element();
        let root_node = root_element.upcast::<Node>();
        root_node
            .traverse_preorder_unrooted(no_gc, ShadowIncluding::No)
            .filter_map(UnrootedDom::downcast::<HTMLLabelElement>)
            .filter(|elem| match elem.GetControl(no_gc) {
                Some(control) => &*control == self,
                _ => false,
            })
            .count() as u32
    }

    // https://html.spec.whatwg.org/multipage/#the-directionality.
    // returns Some if can infer direction by itself or from child nodes
    // returns None if requires to go up to parent
    pub(crate) fn directionality(&self) -> Option<String> {
        let element_direction = &self.Dir();

        if element_direction == "ltr" {
            return Some("ltr".to_owned());
        }

        if element_direction == "rtl" {
            return Some("rtl".to_owned());
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() &&
            matches!(*input.input_type(), InputType::Tel(_))
        {
            return Some("ltr".to_owned());
        }

        if element_direction == "auto" {
            if let Some(directionality) = self
                .downcast::<HTMLInputElement>()
                .and_then(|input| input.auto_directionality())
            {
                return Some(directionality);
            }

            if let Some(area) = self.downcast::<HTMLTextAreaElement>() {
                return Some(area.auto_directionality());
            }
        }

        // TODO(NeverHappened): Implement condition
        // If the element's dir attribute is in the auto state OR
        // If the element is a bdi element and the dir attribute is not in a defined state
        // (i.e. it is not present or has an invalid value)
        // Requires bdi element implementation (https://html.spec.whatwg.org/multipage/#the-bdi-element)

        None
    }

    /// <https://html.spec.whatwg.org/multipage/#the-summary-element:activation-behaviour>
    pub(crate) fn summary_activation_behavior(&self, cx: &mut js::context::JSContext) {
        debug_assert!(self.as_element().local_name() == &local_name!("summary"));

        // Step 1. If this summary element is not the summary for its parent details, then return.
        let is_implicit_summary_element = self.is_implicit_summary_element();
        if !is_implicit_summary_element && !self.is_a_summary_for_its_parent_details() {
            return;
        }

        // Step 2. Let parent be this summary element's parent.
        let parent = if is_implicit_summary_element {
            DomRoot::downcast::<HTMLDetailsElement>(self.containing_shadow_root().unwrap().Host())
                .unwrap()
        } else {
            self.upcast::<Node>()
                .GetParentNode()
                .and_then(DomRoot::downcast::<HTMLDetailsElement>)
                .unwrap()
        };

        // Step 3. If the open attribute is present on parent, then remove it.
        // Otherwise, set parent's open attribute to the empty string.
        parent.toggle(cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#summary-for-its-parent-details>
    pub(crate) fn is_a_summary_for_its_parent_details(&self) -> bool {
        // Step 1. If this summary element has no parent, then return false.
        // Step 2. Let parent be this summary element's parent.
        let Some(parent) = self.upcast::<Node>().GetParentNode() else {
            return false;
        };

        // Step 3. If parent is not a details element, then return false.
        let Some(details) = parent.downcast::<HTMLDetailsElement>() else {
            return false;
        };

        // Step 4. If parent's first summary element child is not this summary
        // element, then return false.
        // Step 5. Return true.
        details
            .find_corresponding_summary_element()
            .is_some_and(|summary| &*summary == self.upcast())
    }

    /// Whether or not this is an implicitly generated `<summary>`
    /// element for a UA `<details>` shadow tree
    fn is_implicit_summary_element(&self) -> bool {
        // Note that non-implicit summary elements are not actually inside
        // the UA shadow tree, they're only assigned to a slot inside it.
        // Therefore they don't cause false positives here
        self.containing_shadow_root()
            .as_deref()
            .map(ShadowRoot::Host)
            .is_some_and(|host| host.is::<HTMLDetailsElement>())
    }

    /// <https://html.spec.whatwg.org/multipage/#rendered-text-fragment>
    fn rendered_text_fragment(
        &self,
        cx: &mut JSContext,
        input: DOMString,
    ) -> DomRoot<DocumentFragment> {
        // Step 1: Let fragment be a new DocumentFragment whose node document is document.
        let document = self.owner_document();
        let fragment = DocumentFragment::new(cx, &document);

        // Step 2: Let position be a position variable for input, initially pointing at the start
        // of input.
        let input = input.str();
        let mut position = input.chars().peekable();

        // Step 3: Let text be the empty string.
        let mut text = String::new();

        // Step 4
        while let Some(ch) = position.next() {
            match ch {
                // While position is not past the end of input, and the code point at position is
                // either U+000A LF or U+000D CR:
                '\u{000A}' | '\u{000D}' => {
                    if ch == '\u{000D}' && position.peek() == Some(&'\u{000A}') {
                        // a \r\n pair should only generate one <br>,
                        // so just skip the \r.
                        position.next();
                    }

                    if !text.is_empty() {
                        append_text_node_to_fragment(cx, &document, &fragment, text);
                        text = String::new();
                    }

                    let br = Element::create(
                        cx,
                        QualName::new(None, ns!(html), local_name!("br")),
                        None,
                        &document,
                        ElementCreator::ScriptCreated,
                        CustomElementCreationMode::Asynchronous,
                        None,
                    );
                    fragment
                        .upcast::<Node>()
                        .AppendChild(cx, br.upcast())
                        .unwrap();
                },
                _ => {
                    // Collect a sequence of code points that are not U+000A LF or U+000D CR from
                    // input given position, and set text to the result.
                    text.push(ch);
                },
            }
        }

        // If text is not the empty string, then append a new Text node whose data is text and node
        // document is document to fragment.
        if !text.is_empty() {
            append_text_node_to_fragment(cx, &document, &fragment, text);
        }

        fragment
    }

    /// Checks whether a given [`DomRoot<Node>`] and its next sibling are
    /// of type [`Text`], and if so merges them into a single [`Text`]
    /// node.
    ///
    /// <https://html.spec.whatwg.org/multipage/#merge-with-the-next-text-node>
    fn merge_with_the_next_text_node(cx: &mut JSContext, node: &Node) {
        // Make sure node is a Text node
        if !node.is::<Text>() {
            return;
        }

        // Step 1: Let next be node's next sibling.
        let next = match node.GetNextSibling() {
            Some(next) => next,
            None => return,
        };

        // Step 2: If next is not a Text node, then return.
        if !next.is::<Text>() {
            return;
        }
        // Step 3: Replace data with node, node's data's length, 0, and next's data.
        let node_chars = node.downcast::<CharacterData>().expect("Node is Text");
        let next_chars = next.downcast::<CharacterData>().expect("Next node is Text");
        node_chars
            .ReplaceData(cx, node_chars.Length(), 0, next_chars.Data())
            .expect("Got chars from Text");

        // Step 4:Remove next.
        next.remove_self(cx);
    }

    /// <https://html.spec.whatwg.org/multipage/#keyboard-shortcuts-processing-model>
    /// > Whenever an element's accesskey attribute is set, changed, or removed, the user agent must
    /// > update the element's assigned access key by running the following steps:
    fn update_assigned_access_key(&self) {
        // 1. If the element has no accesskey attribute, then skip to the fallback step below.
        if !self.element.has_attribute(&local_name!("accesskey")) {
            // This is the same as steps 4 and 5 below.
            self.owner_document()
                .event_handler()
                .unassign_access_key(self);
        }

        // 2. Otherwise, split the attribute's value on ASCII whitespace, and let keys be the resulting tokens.
        let attribute_value = self.element.get_string_attribute(&local_name!("accesskey"));
        let string_view = attribute_value.str();
        let values = string_view.split_html_space_characters();

        // 3. For each value in keys in turn, in the order the tokens appeared in the attribute's
        //    value, run the following substeps:
        for value in values {
            // 1. If the value is not a string exactly one code point in length, then skip the
            //    remainder of these steps for this value.
            let mut characters = value.chars();
            let Some(character) = characters.next() else {
                continue;
            };
            if characters.count() > 0 {
                continue;
            }

            // 2. If the value does not correspond to a key on the system's keyboard, then skip the
            //    remainder of these steps for this value.
            let Some(code) = character_to_code(character) else {
                continue;
            };

            // 3. If the user agent can find a mix of zero or more modifier keys that, combined with
            //    the key that corresponds to the value given in the attribute, can be used as the
            //    access key, then the user agent may assign that combination of keys as the element's
            //    assigned access key and return.
            self.owner_document()
                .event_handler()
                .assign_access_key(self, code);
            return;
        }

        // 4. Fallback: Optionally, the user agent may assign a key combination of its choosing as
        //    the element's assigned access key and then return.
        // We do not do this.

        // 5. If this step is reached, the element has no assigned access key.
        self.owner_document()
            .event_handler()
            .unassign_access_key(self);
    }
}

impl VirtualMethods for HTMLElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.as_element() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut JSContext,
        attr: AttrRef<'_>,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
        let element = self.as_element();
        match (attr.local_name(), mutation) {
            (&local_name!("accesskey"), ..) => {
                self.update_assigned_access_key();
            },
            // https://html.spec.whatwg.org/multipage/#the-popover-attribute
            (&local_name!("popover"), mutation) => {
                // > If element's popover visibility state is in the showing state and oldValue and
                // > value are in different states, then run the hide popover algorithm given
                // > element, true, true, false, and null.
                let old_state = PopoverState::from_attribute_value(mutation.old_value(attr));
                if element.popover_visibility_state_showing() && old_state != self.popover_state() {
                    let _ = self.hide_popover(cx, true, true, false, None);
                }
            },
            // <https://html.spec.whatwg.org/multipage/#the-inert-attribute>
            (&local_name!("inert"), mutation) => {
                // https://html.spec.whatwg.org/multipage/#blocked-by-a-modal-dialog
                // > While document is so blocked, every node that is connected to document, with the
                // > exception of the subject element and its flat tree descendants, must become inert.
                let is_blocked_document = element.is_document_element() &&
                    element
                        .owner_document()
                        .overlay_handler()
                        .is_blocked_by_a_modal_dialog();

                element.set_state(
                    cx.no_gc(),
                    ElementState::INERT,
                    is_blocked_document || !matches!(mutation, AttributeMutation::Removed),
                );
            },
            (&local_name!("form"), mutation) if self.is_form_associated_custom_element() => {
                self.form_attribute_mutated(cx, mutation);
            },
            // Adding or removing a "contenteditable" attribute might make a 0-size selection
            // render its caret or not.
            (&local_name!("contenteditable"), ..) => {
                if let Some(selection) = self.owner_document().selection() &&
                    self.upcast::<Node>()
                        .get_flag(NodeFlags::OVERLAPS_DOCUMENT_SELECTION)
                {
                    selection.set_visible_selection_dirty();
                }
            },
            (&local_name!("disabled"), _) => {
                element.invalidate_disabled_state(cx);
            },
            (&local_name!("readonly"), mutation) if self.is_form_associated_custom_element() => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        element.set_read_write_state(cx.no_gc(), true);
                    },
                    AttributeMutation::Removed => {
                        element.set_read_write_state(cx.no_gc(), false);
                    },
                }
            },
            (&local_name!("nonce"), mutation) => match mutation {
                AttributeMutation::Set(_) => {
                    let nonce = &**attr.value();
                    element.update_nonce_internal_slot(nonce.to_owned(), cx.no_gc());
                },
                AttributeMutation::Removed => {
                    element.update_nonce_internal_slot(String::new(), cx.no_gc());
                },
            },
            // containertiming is not in html5ever, so for now we check manually
            // Layout records these attributes as fragment flags when it builds this element's
            // boxes, so the boxes have to be rebuilt when they change.
            (name, _) if &**name == "containertiming" || &**name == "containertimingignore" => {
                self.upcast::<Node>().dirty(cx.no_gc(), NodeDamage::Other);
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.bind_to_tree(cx, context);
        }
        self.element.invalidate_disabled_state(cx);
        if self.element.has_attribute(&local_name!("accesskey")) {
            self.update_assigned_access_key();
        }
    }

    fn unbind_from_tree(&self, cx: &mut js::context::JSContext, context: &UnbindContext) {
        if let Some(super_type) = self.super_type() {
            super_type.unbind_from_tree(cx, context);
        }
        self.element.invalidate_disabled_state(cx);
        if self.as_element().has_attribute(&local_name!("accesskey")) {
            self.owner_document()
                .event_handler()
                .unassign_access_key(self);
        }

        // https://html.spec.whatwg.org/multipage/#dom-trees
        // > If removedNode's popover attribute is not in the No Popover state, then run the hide
        // > popover algorithm given removedNode, false, false, false, and null.
        if self.popover_state() != PopoverState::NoPopover {
            let _ = self.hide_popover(cx, false, false, false, None);
        }
    }

    fn attribute_affects_presentational_hints(&self, attr: AttrRef<'_>) -> bool {
        if is_element_affected_by_legacy_background_presentational_hint(
            self.element.namespace(),
            self.element.local_name(),
        ) && attr.local_name() == &local_name!("background")
        {
            return true;
        }

        self.super_type()
            .unwrap()
            .attribute_affects_presentational_hints(attr)
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("itemprop") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("itemtype") => AttrValue::from_serialized_tokenlist(value.into()),
            local_name!("background")
                if is_element_affected_by_legacy_background_presentational_hint(
                    self.element.namespace(),
                    self.element.local_name(),
                ) =>
            {
                AttrValue::from_resolved_url(
                    &self.owner_document().base_url().get_arc(),
                    value.into(),
                )
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-trees:html-element-moving-steps>
    fn moving_steps(&self, cx: &mut JSContext, context: &MoveContext) {
        // Step 1. If movedNode is an element whose namespace is the HTML namespace, and this
        // standard defines HTML element moving steps for movedNode's local name, then run the
        // corresponding HTML element moving steps given movedNode.
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(cx, context);
        }

        // Step 2. If movedNode is a form-associated element with a non-null form owner and
        // movedNode and its form owner are no longer in the same tree, then reset the form owner of
        // movedNode.
        if let Some(form_control) = self.element.as_maybe_form_control() {
            form_control.moving_steps(cx);
        }

        self.element.invalidate_disabled_state(cx);
    }
}

impl Activatable for HTMLElement {
    fn as_element(&self) -> &Element {
        &self.element
    }

    fn is_instance_activatable(&self) -> bool {
        self.element.local_name() == &local_name!("summary")
    }

    // Basically used to make the HTMLSummaryElement activatable (which has no IDL definition)
    fn activation_behavior(
        &self,
        cx: &mut js::context::JSContext,
        _event: &Event,
        _target: &EventTarget,
    ) {
        self.summary_activation_behavior(cx);
    }
}

// Form-associated custom elements are the same interface type as
// normal HTMLElements, so HTMLElement needs to have the FormControl trait
// even though it's usually more specific trait implementations, like the
// HTMLInputElement one, that we really want. (Alternately we could put
// the FormControl trait on ElementInternals, but that raises lifetime issues.)
impl FormControl for HTMLElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        debug_assert!(self.is_form_associated_custom_element());
        self.element
            .get_element_internals()
            .and_then(|e| e.form_owner())
    }

    fn set_form_owner(&self, cx: &mut JSContext, form: Option<&HTMLFormElement>) {
        debug_assert!(self.is_form_associated_custom_element());
        self.ensure_element_internals(cx).set_form_owner(form);
    }

    fn to_html_element(&self) -> &HTMLElement {
        self
    }
}
