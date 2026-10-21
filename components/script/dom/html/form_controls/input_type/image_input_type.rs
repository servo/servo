/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref};

use euclid::Point2D;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::codegen::GenericBindings::MouseEventBinding::MouseEventMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::Dom;
use script_bindings::traits::DomEventTrait;
use style_traits::CSSPixel;
use xml5ever::{local_name, ns};

use crate::dom::bindings::root::DomRoot;
use crate::dom::element::AttributeMutation;
use crate::dom::element::attributes::storage::AttrRef;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::embedded_content::htmlimageelement::HTMLImageElement;
use crate::dom::html::form_controls::htmlinputelement::HTMLInputElement;
use crate::dom::html::form_controls::input_type::{SpecificInputActivationType, SpecificInputType};
use crate::dom::htmlformelement::{FormControl, FormSubmitterElement, SubmittedFrom};
use crate::dom::input_type::InputType;
use crate::dom::input_type::text_input_widget::TextInputWidget;
use crate::dom::node::NodeTraits;
use crate::dom::shadowroot::shadowroot::ShadowRoot;
use crate::dom::shadowroot::ua_shadowroot::{SpecificShadowTree, UAShadowRoot};
use crate::dom::types::MouseEvent;
use crate::dom::{Element, Node};

#[derive(Default, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ImageInputType {
    shadow_tree: DomRefCell<Option<ImageInputShadowTree>>,
    /// <https://html.spec.whatwg.org/multipage/#concept-input-type-image-coordinate>
    #[no_trace]
    selected_coordinate: Cell<Point2D<i32, CSSPixel>>,
}

#[derive(Clone, Copy)]
pub(crate) struct ImageInputActivation;

impl SpecificInputType for ImageInputType {
    fn text_input_widget(&self) -> Option<&DomRefCell<TextInputWidget>> {
        None
    }

    fn update_shadow_tree(&self, cx: &mut JSContext, input: &HTMLInputElement) {
        self.ensure_shadow_tree(cx, input.upcast())
            .update(cx, self, input)
    }

    fn attribute_mutated(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        attr: AttrRef<'_>,
        _: AttributeMutation,
    ) {
        if matches!(*attr.local_name(), local_name!("src") | local_name!("alt")) {
            self.update_shadow_tree(cx, input);
        }
    }
}

impl ImageInputType {
    pub(crate) fn set_selected_coordinate(&self, input: &HTMLInputElement, event: &Event) {
        if !event.IsTrusted() {
            return;
        }
        let Some(mouse_event) = event.downcast::<MouseEvent>() else {
            return;
        };
        let rect = input.upcast::<Node>().client_rect();
        self.selected_coordinate.set(Point2D::new(
            mouse_event.ClientX() - rect.origin.x,
            mouse_event.ClientY() - rect.origin.y,
        ));
    }

    pub(crate) fn selected_coordinate(&self) -> Point2D<i32, CSSPixel> {
        self.selected_coordinate.get()
    }

    /// Step 2 and later of <https://html.spec.whatwg.org/multipage/#input-img-dimensions>
    pub(crate) fn dimensions(&self, input: &HTMLInputElement) -> (u32, u32) {
        // Step 2. Let naturalDimensions be null.
        // Step 3. If an image is available,
        // then set naturalDimensions to the result of applying the default sizing algorithm
        // with that image's natural width, natural height, and natural aspect ratio,
        // if any, using a default object size of 300 by 150.
        let natural_dimensions = self
            .borrow_for_shadow_tree()
            .as_ref()
            .and_then(|shadow_tree| {
                shadow_tree.image_element.image_data().map(|image_data| {
                    let metadata = image_data.metadata();
                    (metadata.width, metadata.height)
                })
            });
        // Step 4. Return the result of determining the dimensions of input given naturalDimensions.
        HTMLImageElement::determine_dimensions(input.upcast(), natural_dimensions)
    }
}

impl SpecificInputActivationType for ImageInputActivation {
    /// <https://html.spec.whatwg.org/multipage/#image-button-state-(type=image):input-activation-behavior>
    fn activation_behavior(
        &self,
        cx: &mut JSContext,
        input: &HTMLInputElement,
        event: &Event,
        _target: &EventTarget,
    ) {
        // Step 1: If the element does not have a form owner, then return.
        if let Some(form_owner) = input.form_owner() {
            let document = input.owner_document();

            // Step 2: If the element's node document is not fully active, then return.
            if !document.is_fully_active() {
                return;
            }

            // Step 3: If the user activated the control while explicitly selecting a coordinate,
            // then set the element's selected coordinate to that coordinate.
            if let InputType::Image(ref image_input_type) = *input.input_type() {
                image_input_type.set_selected_coordinate(input, event);
            }

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
pub(crate) struct ImageInputShadowTree {
    image_element: Dom<HTMLImageElement>,
}

impl ImageInputShadowTree {
    pub(crate) fn new(cx: &mut JSContext, shadow_root: &Node) -> Self {
        let document = shadow_root.owner_document();
        let img = Self::create_element_in_ua_shadowroot(cx, &document, local_name!("img"));
        Node::replace_all(cx, Some(img.upcast()), shadow_root);
        let img = img
            .downcast::<HTMLImageElement>()
            .expect("Must always create an image element");
        Self {
            image_element: Dom::from_ref(img),
        }
    }
}

impl SpecificShadowTree<HTMLInputElement, ImageInputType> for ImageInputShadowTree {
    fn update(&self, cx: &mut JSContext, _: &ImageInputType, input_element: &HTMLInputElement) {
        let input_element = input_element.upcast::<Element>();
        for name in [&local_name!("src"), &local_name!("alt")] {
            let new = input_element.get_attribute_string_value(name);
            let image_element = self.image_element.upcast::<Element>();
            if new == image_element.get_attribute_string_value(name) {
                continue;
            }
            match new {
                Some(value) => image_element.set_attribute(cx, name, value.into()),
                None => {
                    image_element.remove_attribute(cx, &ns!(), name);
                },
            }
        }
    }
}

impl UAShadowRoot<ImageInputShadowTree> for ImageInputType {
    fn store_for_shadow_tree(&self, cx: &mut JSContext, shadow_root: DomRoot<ShadowRoot>) {
        *self.shadow_tree.safe_borrow_mut(cx.no_gc()) =
            Some(ImageInputShadowTree::new(cx, shadow_root.upcast()));
    }

    fn borrow_for_shadow_tree(&self) -> Ref<'_, Option<ImageInputShadowTree>> {
        self.shadow_tree.borrow()
    }
}
