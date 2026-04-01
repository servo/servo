/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::codegen::GenericBindings::ElementBinding::ElementMethods;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::codegen::InheritTypes::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use script_bindings::inheritance::Castable;
use style::computed_values::visibility::T as Visibility;
use style::values::computed::Overflow;
use xml5ever::local_name;

use crate::dom::Node;
use crate::dom::document::focus::FocusableAreaKind;
use crate::dom::types::{Element, HTMLElement};

impl Element {
    /// <https://html.spec.whatwg.org/multipage/#focusable-area>
    ///
    /// The list of focusable areas at this point in the specification is both incomplete and leaves
    /// a lot up to the user agent. In addition, the specifications for "click focusable" and
    /// "sequentially focusable" are written in a way that they are subsets of all focusable areas.
    /// In order to avoid having to first determine whether an element is a focusable area and then
    /// work backwards to figure out what kind it is, this function attempts to classify the
    /// different types of focusable areas ahead of time so that the logic is useful for answering
    /// both "Is this element a focusable area?" and "Is this element click (or sequentially)
    /// focusable."
    pub(crate) fn focusable_area_kind(&self) -> FocusableAreaKind {
        // Do not allow unrendered, disconnected, or disabled nodes to be focusable areas ever.
        let node: &Node = self.upcast();
        if !node.is_connected() || !self.has_css_layout_box() || self.is_actually_disabled() {
            return Default::default();
        }

        // <https://www.w3.org/TR/css-display-4/#visibility>
        // Invisible elements are removed from navigation.
        if self
            .style()
            .is_some_and(|style| style.get_inherited_box().visibility != Visibility::Visible)
        {
            return Default::default();
        }

        // An element with a shadow root that delegates focus should never itself be a focusable area.
        if self
            .shadow_root()
            .is_some_and(|shadow_root| shadow_root.DelegatesFocus())
        {
            return Default::default();
        }

        // > Elements that meet all the following criteria:
        // > the element's tabindex value is non-null, or the element is determined by the user agent to be focusable;
        // > the element is either not a shadow host, or has a shadow root whose delegates focus is false;
        // Note: Checked above
        // > the element is not actually disabled;
        // Note: Checked above
        // > the element is not inert;
        // TODO: Handle this.
        // > the element is either being rendered, delegating its rendering to its children, or
        // > being used as relevant canvas fallback content.
        // Note: Checked above
        // TODO: Handle fallback canvas content.
        match self.explicitly_set_tab_index() {
            // From <https://html.spec.whatwg.org/multipage/#tabindex-ordered-focus-navigation-scope>:
            // > A tabindex-ordered focus navigation scope is a list of focusable areas and focus
            // > navigation scope owners. Every focus navigation scope owner owner has tabindex-ordered
            // > focus navigation scope, whose contents are determined as follows:
            // >  - It contains all elements in owner's focus navigation scope that are themselves focus
            // >    navigation scope owners, except the elements whose tabindex value is a negative integer.
            // >  - It contains all of the focusable areas whose DOM anchor is an element in owner's focus
            // >    navigation scope, except the focusable areas whose tabindex value is a negative integer.
            Some(tab_index) if tab_index < 0 => return FocusableAreaKind::Click,
            Some(_) => return FocusableAreaKind::Click | FocusableAreaKind::Sequential,
            None => {},
        }

        // From <https://html.spec.whatwg.org/multipage/#tabindex-value>
        // > If the value is null
        // > ...
        // > Modulo platform conventions, it is suggested that the following elements should be
        // > considered as focusable areas and be sequentially focusable:
        let is_focusable_area_due_to_type = match node.type_id() {
            // >  - a elements that have an href attribute
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) => self.has_attribute(&local_name!("href")),

            // >  - input elements whose type attribute are not in the Hidden state
            // >  - button elements
            // >  - select elements
            // >  - textarea elements
            // >  - Navigable containers
            //
            // Note: the `hidden` attribute is checked above for all elements.
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLInputElement |
                HTMLElementTypeId::HTMLButtonElement |
                HTMLElementTypeId::HTMLSelectElement |
                HTMLElementTypeId::HTMLTextAreaElement |
                HTMLElementTypeId::HTMLIFrameElement,
            )) => true,
            _ => {
                // >  - summary elements that are the first summary element child of a details element
                // >  - Editing hosts
                // > -  Elements with a draggable attribute set, if that would enable the user agent to allow
                // >    the user to begin drag operations for those elements without the use of a pointing device
                self.downcast::<HTMLElement>()
                    .is_some_and(|html_element| html_element.is_a_summary_for_its_parent_details()) ||
                    self.is_editing_host() ||
                    self.get_string_attribute(&local_name!("draggable")) == "true"
            },
        };

        if is_focusable_area_due_to_type {
            return FocusableAreaKind::Click | FocusableAreaKind::Sequential;
        }

        // > The scrollable regions of elements that are being rendered and are not inert.
        //
        // Note that these kind of focusable areas are only focusable via the keyboard.
        //
        // TODO: Handle inert.
        if self
            .upcast::<Node>()
            .effective_overflow()
            .is_some_and(|axes_overflow| {
                // This is checking whether there is an input event scrollable overflow value in
                // a given axis and also overflow in that same axis.
                (matches!(axes_overflow.x, Overflow::Auto | Overflow::Scroll) &&
                    self.ScrollWidth() > self.ClientWidth()) ||
                    (matches!(axes_overflow.y, Overflow::Auto | Overflow::Scroll) &&
                        self.ScrollHeight() > self.ClientHeight())
            })
        {
            return FocusableAreaKind::Sequential;
        }

        Default::default()
    }

    /// <https://html.spec.whatwg.org/multipage/#sequentially-focusable>.
    pub(crate) fn is_sequentially_focusable(&self) -> bool {
        self.focusable_area_kind()
            .contains(FocusableAreaKind::Sequential)
    }

    /// <https://html.spec.whatwg.org/multipage/#focusable-area>
    pub(crate) fn is_focusable_area(&self) -> bool {
        !self.focusable_area_kind().is_empty()
    }
}
