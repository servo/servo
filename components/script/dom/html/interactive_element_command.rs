/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::codegen::GenericBindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use script_bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElementMethods;
use script_bindings::codegen::GenericBindings::HTMLInputElementBinding::HTMLInputElementMethods;
use script_bindings::codegen::GenericBindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use script_bindings::codegen::GenericBindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::document::FocusInitiator;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::types::{
    HTMLAnchorElement, HTMLButtonElement, HTMLElement, HTMLFieldSetElement, HTMLInputElement,
    HTMLLabelElement, HTMLLegendElement, HTMLOptionElement,
};

/// This is an implementation of <https://html.spec.whatwg.org/multipage/#concept-command>. Note
/// that there are various things called "commands" on the web platform, but this is the one that
/// is mainly associated with access keys.
pub(crate) enum InteractiveElementCommand {
    Anchor(DomRoot<HTMLAnchorElement>),
    Button(DomRoot<HTMLButtonElement>),
    Input(DomRoot<HTMLInputElement>),
    Option(DomRoot<HTMLOptionElement>),
    HTMLElement(DomRoot<HTMLElement>),
}

impl TryFrom<&HTMLLegendElement> for InteractiveElementCommand {
    type Error = ();

    /// From <https://html.spec.whatwg.org/multipage/#using-the-accesskey-attribute-on-a-legend-element-to-define-a-command>
    /// A legend element defines a command if all of the following are true:
    ///  - It has an assigned access key.
    ///  - It is a child of a fieldset element.
    ///  - Its parent has a descendant that defines a command that is neither a label element nor
    ///    a legend element. This element, if it exists, is the legend element's accesskey
    ///    delegatee.
    fn try_from(legend_element: &HTMLLegendElement) -> Result<Self, Self::Error> {
        if !legend_element
            .owner_document()
            .event_handler()
            .has_assigned_access_key(legend_element.upcast())
        {
            return Err(());
        }

        let node = legend_element.upcast::<Node>();
        let Some(parent) = node.GetParentElement() else {
            return Err(());
        };
        if !parent.is::<HTMLFieldSetElement>() {
            return Err(());
        }
        for node in parent
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
        {
            if node.is::<HTMLLabelElement>() || node.is::<HTMLLegendElement>() {
                continue;
            }
            let Some(html_element) = node.downcast::<HTMLElement>() else {
                continue;
            };
            if let Ok(command) = Self::try_from(html_element) {
                return Ok(command);
            }
        }

        Err(())
    }
}

impl TryFrom<&HTMLElement> for InteractiveElementCommand {
    type Error = ();

    fn try_from(html_element: &HTMLElement) -> Result<Self, Self::Error> {
        if let Some(anchor_element) = html_element.downcast::<HTMLAnchorElement>() {
            return Ok(Self::Anchor(DomRoot::from_ref(anchor_element)));
        }
        if let Some(button_element) = html_element.downcast::<HTMLButtonElement>() {
            return Ok(Self::Button(DomRoot::from_ref(button_element)));
        }
        if let Some(input_element) = html_element.downcast::<HTMLInputElement>() {
            return Ok(Self::Input(DomRoot::from_ref(input_element)));
        }
        if let Some(option_element) = html_element.downcast::<HTMLOptionElement>() {
            return Ok(Self::Option(DomRoot::from_ref(option_element)));
        }
        if let Some(legend_element) = html_element.downcast::<HTMLLegendElement>() {
            return Self::try_from(legend_element);
        }
        if html_element
            .owner_document()
            .event_handler()
            .has_assigned_access_key(html_element)
        {
            return Ok(Self::HTMLElement(DomRoot::from_ref(html_element)));
        }

        Err(())
    }
}

impl InteractiveElementCommand {
    pub(crate) fn disabled(&self) -> bool {
        match self {
            // <https://html.spec.whatwg.org/multipage#using-the-a-element-to-define-a-command>
            // > The Disabled State facet of the command is true if the element or one of its
            // > ancestors is inert, and false otherwise.
            // TODO: We do not support `inert` yet.
            InteractiveElementCommand::Anchor(..) => false,
            // <https://html.spec.whatwg.org/multipage/#using-the-button-element-to-define-a-command>
            // > The Disabled State of the command is true if the element or one of its ancestors
            // > is inert, or if the element's disabled state is set, and false otherwise.
            // TODO: We do not support `inert` yet.
            InteractiveElementCommand::Button(button) => button.Disabled(),
            // <https://html.spec.whatwg.org/multipage/#using-the-input-element-to-define-a-command>
            // > The Disabled State of the command is true if the element or one of its ancestors is
            // > inert, or if the element's disabled state is set, and false otherwise.
            // TODO: We do not support `inert` yet.
            InteractiveElementCommand::Input(input) => input.Disabled(),
            // <https://html.spec.whatwg.org/multipage/#using-the-option-element-to-define-a-command>
            // > The Disabled State of the command is true if the element is disabled, or if its
            // > nearest ancestor select element is disabled, or if it or one of its ancestors is
            // > inert, and false otherwise.
            // TODO: We do not support `inert` yet.
            InteractiveElementCommand::Option(option) => {
                option.Disabled() ||
                    option
                        .nearest_ancestor_select()
                        .is_some_and(|select| select.Disabled())
            },
            // <https://html.spec.whatwg.org/multipage#using-the-accesskey-attribute-to-define-a-command-on-other-elements>
            // > The Disabled State of the command is true if the element or one of its ancestors is
            // > inert, and false otherwise.
            // TODO: We do not support `inert` yet.
            InteractiveElementCommand::HTMLElement(..) => false,
        }
    }

    pub(crate) fn hidden(&self) -> bool {
        let html_element: &HTMLElement = match self {
            InteractiveElementCommand::Anchor(anchor_element) => anchor_element.upcast(),
            InteractiveElementCommand::Button(button_element) => button_element.upcast(),
            InteractiveElementCommand::Input(input_element) => input_element.upcast(),
            InteractiveElementCommand::Option(option_element) => option_element.upcast(),
            InteractiveElementCommand::HTMLElement(html_element) => html_element,
        };
        html_element.Hidden()
    }

    pub(crate) fn perform_action(&self, can_gc: CanGc) {
        match self {
            // <https://html.spec.whatwg.org/multipage#using-the-a-element-to-define-a-command>
            // > The Action of the command is to fire a click event at the element.
            // <https://html.spec.whatwg.org/multipage/#fire-a-click-event>
            // > Firing a click event at target means firing a synthetic pointer event named click at target.
            InteractiveElementCommand::Anchor(anchor_element) => anchor_element
                .upcast::<Node>()
                .fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc),
            // <https://html.spec.whatwg.org/multipage/#using-the-button-element-to-define-a-command>
            // > The Label, Access Key, Hidden State, and Action facets of the command are
            // > determined as for a elements (see the previous section).
            InteractiveElementCommand::Button(button_element) => button_element
                .upcast::<Node>()
                .fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc),
            // <https://html.spec.whatwg.org/multipage/#using-the-input-element-to-define-a-command>
            // > The Action of the command is to fire a click event at the element.
            InteractiveElementCommand::Input(input_element) => input_element
                .upcast::<Node>()
                .fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc),
            // <https://html.spec.whatwg.org/multipage/#using-the-option-element-to-define-a-command>
            // > If the option's nearest ancestor select element has a multiple attribute, the
            // > Action of the command is to toggle the option element. Otherwise, the Action is to
            // > pick the option element.
            // Note: setSelected takes care of whether or not the owner has the `multiple` attribute.
            InteractiveElementCommand::Option(option_element) => {
                option_element.SetSelected(true, can_gc)
            },
            // > The Action of the command is to run the following steps:
            //    > 1. Run the focusing steps for the element.
            //    > 2. Fire a click event at the element.
            InteractiveElementCommand::HTMLElement(html_element) => {
                html_element.owner_document().request_focus(
                    Some(html_element.upcast()),
                    FocusInitiator::Script,
                    can_gc,
                );
                html_element
                    .upcast::<Node>()
                    .fire_synthetic_pointer_event_not_trusted(atom!("click"), can_gc);
            },
        }
    }
}
