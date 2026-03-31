/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use bitflags::bitflags;
use script_bindings::codegen::GenericBindings::ElementBinding::ElementMethods;
use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::codegen::InheritTypes::{ElementTypeId, HTMLElementTypeId, NodeTypeId};
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;
use style::computed_values::visibility::T as Visibility;
use style::values::computed::Overflow;
use xml5ever::local_name;

use crate::dom::bindings::codegen::Bindings::HTMLOrSVGElementBinding::FocusOptions;
use crate::dom::types::{Element, HTMLDialogElement, HTMLElement};
use crate::dom::{FocusInitiator, Node, NodeTraits, ShadowIncluding};

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
    fn focusable_area_kind(&self) -> FocusableAreaKind {
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

    /// <https://html.spec.whatwg.org/multipage/#click-focusable>
    pub(crate) fn is_click_focusable(&self) -> bool {
        self.focusable_area_kind()
            .contains(FocusableAreaKind::Click)
    }

    /// <https://html.spec.whatwg.org/multipage/#focusable-area>
    pub(crate) fn is_focusable_area(&self) -> bool {
        !self.focusable_area_kind().is_empty()
    }

    /// Returns the focusable appropriate DOM anchor for the focuable area when this element is
    /// clicked on according to <https://www.w3.org/TR/pointerevents4/#handle-native-mouse-down>.
    ///
    /// Note that this is doing more than the specification which says to only take into account
    /// the node from the hit test. This isn't exactly how browsers work though, as they seem
    /// to look for the first inclusive ancestor node that has a focusable area associated with it.
    pub(crate) fn find_click_focusable_area(&self) -> Option<DomRoot<Element>> {
        Some(
            self.upcast::<Node>()
                .inclusive_ancestors(ShadowIncluding::Yes)
                .find_map(|node| {
                    DomRoot::downcast::<Element>(node)
                        .iter()
                        .filter_map(|element| element.get_the_focusable_area())
                        .find(|(_, focusable_area_kind)| {
                            focusable_area_kind.contains(FocusableAreaKind::Click)
                        })
                })?
                .0,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-focusable-area>
    ///
    /// There seems to be hole in the specification here. It describes how to get the focusable
    /// area for a focus target that isn't a focuable area, but is ambiguous about how to do
    /// this for a focus target that actually *is* a focusable area. The obvious thing is to
    /// just return the focus target, but it's still odd that this isn't mentioned in the
    /// specification.
    fn get_the_focusable_area(&self) -> Option<(DomRoot<Element>, FocusableAreaKind)> {
        let focusable_area_kind = self.focusable_area_kind();
        if !focusable_area_kind.is_empty() {
            return Some((DomRoot::from_ref(self), focusable_area_kind));
        }
        self.get_the_focusable_area_if_not_a_focusable_area()
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-focusable-area>
    ///
    /// In addition to returning the DOM anchor of the focusable area for this [`Element`], this
    /// method also returns the [`FocusableAreaKind`] for efficiency reasons. Note that `None`
    /// is returned if this [`Element`] does not have a focusable area or if its focusable area
    /// is the `Document`'s viewport.
    ///
    /// TODO: It might be better to distinguish these two cases in the future.
    fn get_the_focusable_area_if_not_a_focusable_area(
        &self,
    ) -> Option<(DomRoot<Element>, FocusableAreaKind)> {
        // > To get the focusable area for a focus target that is either an element that is not a
        // > focusable area, or is a navigable, given an optional string focus trigger (default
        // > "other"), run the first matching set of steps from the following list:
        //
        // > ↪ If focus target is an area element with one or more shapes that are focusable areas
        // >     Return the shape corresponding to the first img element in tree order that uses the image
        // >     map to which the area element belongs.
        // TODO: Implement this.

        // > ↪ If focus target is an element with one or more scrollable regions that are focusable areas
        // >     Return the element's first scrollable region, according to a pre-order, depth-first
        // >     traversal of the flat tree. [CSSSCOPING]
        // TODO: Implement this.

        // > ↪ If focus target is the document element of its Document
        // >     Return the Document's viewport.
        // TODO: Implement this.

        // > ↪ If focus target is a navigable
        // >     Return the navigable's active document.
        // TODO: Implement this.

        // > ↪ If focus target is a navigable container with a non-null content navigable
        // >     Return the navigable container's content navigable's active document.
        // TODO: Implement this.

        // > ↪ If focus target is a shadow host whose shadow root's delegates focus is true
        // >     1. Let focusedElement be the currently focused area of a top-level traversable's DOM
        // >        anchor.
        // >     2. If focus target is a shadow-including inclusive ancestor of focusedElement, then
        // >        return focusedElement.
        // >     3. Return the focus delegate for focus target given focus trigger.
        if self
            .shadow_root()
            .is_some_and(|shadow_root| shadow_root.DelegatesFocus())
        {
            if let Some(focused_element) = self.owner_document().get_focused_element() {
                if self
                    .upcast::<Node>()
                    .is_shadow_including_inclusive_ancestor_of(focused_element.upcast())
                {
                    let focusable_area_kind = focused_element.focusable_area_kind();
                    return Some((focused_element, focusable_area_kind));
                }
            }
            return self.focus_delegate();
        }

        None
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-delegate>
    ///
    /// In addition to returning the focus delegate for this [`Element`], this method also returns
    /// the [`FocusableAreaKind`] for efficiency reasons.
    fn focus_delegate(&self) -> Option<(DomRoot<Element>, FocusableAreaKind)> {
        // > 1. If focusTarget is a shadow host and its shadow root's delegates focus is false, then
        // >    return null.
        let shadow_root = self.shadow_root();
        if shadow_root
            .as_ref()
            .is_some_and(|shadow_root| !shadow_root.DelegatesFocus())
        {
            return None;
        }

        // > 2. Let whereToLook be focusTarget.
        let mut where_to_look = self.upcast::<Node>();

        // > 3. If whereToLook is a shadow host, then set whereToLook to whereToLook's shadow root.
        if let Some(shadow_root) = shadow_root.as_ref() {
            where_to_look = shadow_root.upcast();
        }

        // > 4. Let autofocusDelegate be the autofocus delegate for whereToLook given focusTrigger.
        // TODO: Implement this.

        // > 5. If autofocusDelegate is not null, then return autofocusDelegate.
        // TODO: Implement this.

        // > 6. For each descendant of whereToLook's descendants, in tree order:
        let is_dialog_element = self.is::<HTMLDialogElement>();
        for descendant in where_to_look.traverse_preorder(ShadowIncluding::No).skip(1) {
            // > 6.1. Let focusableArea be null.
            // Handled via early return.

            let Some(descendant) = descendant.downcast::<Element>() else {
                continue;
            };

            // > 6.2. If focusTarget is a dialog element and descendant is sequentially focusable, then
            // >      set focusableArea to descendant.
            let focusable_area_kind = descendant.focusable_area_kind();
            if is_dialog_element && focusable_area_kind.contains(FocusableAreaKind::Sequential) {
                return Some((DomRoot::from_ref(descendant), focusable_area_kind));
            }

            // > 6.3. Otherwise, if focusTarget is not a dialog and descendant is a focusable area, set
            // >      focusableArea to descendant.
            if !focusable_area_kind.is_empty() {
                return Some((DomRoot::from_ref(descendant), focusable_area_kind));
            }

            // > 6.4. Otherwise, set focusableArea to the result of getting the focusable area for
            //        descendant given focusTrigger.
            if let Some(focusable_area) =
                descendant.get_the_focusable_area_if_not_a_focusable_area()
            {
                // > 6.5. If focusableArea is not null, then return focusableArea.
                return Some(focusable_area);
            }
        }

        // > 7. Return null.
        None
    }

    /// <https://html.spec.whatwg.org/multipage/#focusing-steps>
    ///
    /// This is an initial implementation of the "focusing steps" from the HTML specification. Note
    /// that this is currently in a state of transition from Servo's old internal focus APIs to ones
    /// that match the specification. That is why the arguments to this method do not match the
    /// specification yet.
    pub(crate) fn run_the_focusing_steps(
        &self,
        focus_initiator: FocusInitiator,
        focus_options: FocusOptions,
        can_gc: CanGc,
    ) {
        // > 1. If new focus target is not a focusable area, then set new focus target to the result
        // >    of getting the focusable area for new focus target, given focus trigger if it was
        // >    passed.
        let Some((element, _)) = self.get_the_focusable_area() else {
            return;
        };

        // > 2. If new focus target is null, then:
        // > 2.1 If no fallback target was specified, then return.
        // > 2.2 Otherwise, set new focus target to the fallback target.
        // TODO: Handle the fallback.

        // > 3. If new focus target is a navigable container with non-null content navigable, then
        // >    set new focus target to the content navigable's active document.
        // > 4. If new focus target is a focusable area and its DOM anchor is inert, then return.
        // > 5. If new focus target is the currently focused area of a top-level traversable, then
        // >    return.
        // > 6. Let old chain be the current focus chain of the top-level traversable in which new
        // >    focus target finds itself.
        // > 6.1. Let new chain be the focus chain of new focus target.
        // > 6.2. Run the focus update steps with old chain, new chain, and new focus target
        // >      respectively.
        //
        // TODO: Handle all of these steps by converting the focus transaction code to follow
        // the HTML focus specification.
        let document = self.owner_document();
        document.request_focus_with_options(
            Some(&*element),
            focus_initiator,
            focus_options,
            can_gc,
        );
    }
}

/// What kind of focusable area an [`Element`] is.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FocusableAreaKind(u8);

bitflags! {
    impl FocusableAreaKind: u8 {
        /// <https://html.spec.whatwg.org/multipage/#click-focusable>
        ///
        /// > A focusable area is said to be click focusable if the user agent determines that it is
        /// > click focusable. User agents should consider focusable areas with non-null tabindex values
        /// > to be click focusable.
        const Click = 1 << 0;
        /// <https://html.spec.whatwg.org/multipage/#sequentially-focusable>.
        ///
        /// > A focusable area is said to be sequentially focusable if it is included in its
        /// > Document's sequential focus navigation order and the user agent determines that it is
        /// > sequentially focusable.
        const Sequential = 1 << 1;
    }
}
