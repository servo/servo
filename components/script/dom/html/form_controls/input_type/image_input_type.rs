use std::cell::Ref;

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::Dom;
use xml5ever::{QualName, local_name, ns};

use crate::dom::{CustomElementCreationMode, Element, ElementCreator, Node};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::form_controls::input_type::text_value_widget::TextValueWidget;
use crate::dom::html::form_controls::input_type::{SpecificInputActivationType, SpecificInputType};
use crate::dom::htmlformelement::{FormControl, FormSubmitterElement, SubmittedFrom};
use crate::dom::input_type::text_input_widget::TextInputWidget;
use crate::dom::node::NodeTraits;


#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ImageInputType {
    text_value_widget: DomRefCell<TextValueWidget>,
    shadow_tree: DomRefCell<Option<ImageInputShadowTree>>,
}

#[derive(Clone, Copy)]
pub(crate) struct ImageInputActivation;

impl SpecificInputType for ImageInputType {
    fn text_input_widget(&self) -> Option<&DomRefCell<TextInputWidget>> {
        None
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.get_or_create_shadow_tree(cx, input).update(cx, input)
    }
}

impl ImageInputType {
    fn get_or_create_shadow_tree(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
    ) -> Ref<'_, ImageInputShadowTree> {
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
        *self.shadow_tree.borrow_mut() = Some(ImageInputShadowTree::new(cx, shadow_root));
        self.get_or_create_shadow_tree(cx, input)
    }
}

impl SpecificInputActivationType for ImageInputActivation {
    /// <https://html.spec.whatwg.org/multipage/#image-button-state-(type=image):input-activation-behavior>
    fn activation_behavior(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        _event: &Event,
        _target: &EventTarget,
    ) {
        // Step 1: If the element does not have a form owner, then return.
        if let Some(form_owner) = input.form_owner() {
            let document = input.owner_document();

            // Step 2: If the element's node document is not fully active, then return.
            if !document.is_fully_active() {
                return;
            }

            // TODO Step 3. If the user activated the control while explicitly selecting a coordinate,
            // then set the element's selected coordinate to that coordinate.

            // Step 4: Submit the element's form owner from the element with userInvolvement
            // set to event's user navigation involvement.
            form_owner.submit(
                cx,
                SubmittedFrom::NotFromForm,
                FormSubmitterElement::Input(input),
            )
        }
    }
}

#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ImageInputShadowTree{
    image_element: Dom<Element>
}

impl ImageInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node,) -> Self {
        let img = Element::create(
            cx,
            QualName::new(None, ns!(html), local_name!("img")),
            None,
            &shadow_root.owner_document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Asynchronous,
            None,
        );
        
        let _ = shadow_root.AppendChild(cx, img.upcast());
        Self {
            image_element: img.as_traced()
        }
    }

    pub(crate) fn update(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        let input_element = input.upcast::<Element>();
        for name in [&local_name!("src"), &local_name!("alt")] {
            let new = input_element.get_attribute_string_value(name);
            if new == self.image_element.get_attribute_string_value(name) {
                continue;
            }
            match new {
                Some(value) => self
                    .image_element
                    .set_attribute(cx, name, value.into()),
                None => {
                    self.image_element.remove_attribute(cx, &ns!(), name);
                },
            }
        } 
    }
}
