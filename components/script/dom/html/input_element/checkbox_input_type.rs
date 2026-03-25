/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::Ref;

use html5ever::{local_name, ns};
use js::context::JSContext;
use markup5ever::QualName;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::domstring::DOMString;
use script_bindings::inheritance::Castable;
use script_bindings::root::Dom;
use script_bindings::script_runtime::CanGc;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventComposed};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlinputelement::text_value_widget::TextValueWidget;
use crate::dom::input_element::input_type::SpecificInputType;
use crate::dom::input_element::{HTMLInputElement, InputActivationState};
use crate::dom::node::{Node, NodeTraits};

const CHECKMARK_CONTAINER_STYLE: &str = "
    font-size: 12px;
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
";

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct CheckboxInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
    shadow_tree: DomRefCell<Option<CheckboxInputShadowTree>>,
}

impl CheckboxInputType {
    /// Get the shadow tree for this [`HTMLInputElement`], if it is created and valid, otherwise
    /// recreate the shadow tree and return it.
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, CheckboxInputShadowTree> {
        {
            if let Ok(shadow_tree) = Ref::filter_map(self.shadow_tree.borrow(), |shadow_tree| {
                shadow_tree.as_ref()
            }) {
                return shadow_tree;
            }
        }

        let element = input.upcast::<Element>();
        let shadow_root = element
            .shadow_root()
            .unwrap_or_else(|| element.attach_ua_shadow_root(cx, true));
        let shadow_root = shadow_root.upcast();
        *self.shadow_tree.borrow_mut() = Some(CheckboxInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }
}

impl SpecificInputType for CheckboxInputType {
    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):suffering-from-being-missing>
    fn suffers_from_being_missing(&self, input: &HTMLInputElement, _value: &DOMString) -> bool {
        input.Required() && !input.Checked()
    }

    /// <https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):input-activation-behavior>
    fn activation_behavior(
        &self,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
        can_gc: CanGc,
    ) {
        // Step 1: If the element is not connected, then return.
        if !input.upcast::<Node>().is_connected() {
            return;
        }

        let target = input.upcast::<EventTarget>();

        // Step 2: Fire an event named input at the element with the bubbles and composed
        // attributes initialized to true.
        target.fire_event_with_params(
            atom!("input"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            can_gc,
        );

        // Step 3: Fire an event named change at the element with the bubbles attribute
        // initialized to true.
        target.fire_bubbling_event(atom!("change"), can_gc);
    }

    /// <https://html.spec.whatwg.org/multipage/#the-input-element:legacy-pre-activation-behavior>
    fn legacy_pre_activation_behavior(
        &self,
        input: &HTMLInputElement,
        can_gc: CanGc,
    ) -> Option<InputActivationState> {
        let was_checked = input.Checked();
        let was_indeterminate = input.Indeterminate();
        input.SetIndeterminate(false);
        input.SetChecked(!was_checked, can_gc);
        Some(InputActivationState {
            checked: was_checked,
            indeterminate: was_indeterminate,
            checked_radio: None,
            was_radio: false,
            was_checkbox: true,
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#the-input-element:legacy-canceled-activation-behavior>
    fn legacy_canceled_activation_behavior(
        &self,
        input: &HTMLInputElement,
        cache: InputActivationState,
        can_gc: CanGc,
    ) {
        input.SetIndeterminate(cache.indeterminate);
        input.SetChecked(cache.checked, can_gc);
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(
            cx,
            input.Checked(),
            input.Indeterminate(),
        );
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
/// Contains references to the elements in the shadow tree for `<input type=file>`.
///
/// The shadow tree consists of the file selector button and a span for the chosen files text.
pub(crate) struct CheckboxInputShadowTree {
    checkmark_container: Dom<Element>,
}

impl CheckboxInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let checkmark_container = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("span")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );

        checkmark_container.set_string_attribute(
            &local_name!("style"),
            CHECKMARK_CONTAINER_STYLE.into(),
            CanGc::from_cx(cx),
        );

        shadow_root
            .upcast::<Node>()
            .AppendChild(cx, checkmark_container.upcast())
            .unwrap();

        Self {
            checkmark_container: checkmark_container.as_traced(),
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, checked: bool, indeterminate: bool) {
        self.checkmark_container
            .upcast::<Node>()
            .set_text_content_for_element(cx, Some("".into()));

        if checked {
            self.checkmark_container
                .upcast::<Node>()
                .set_text_content_for_element(cx, Some("✔".into()));
        }
        if indeterminate {
            self.checkmark_container
                .upcast::<Node>()
                .set_text_content_for_element(cx, Some("—".into()));
        }
    }
}
