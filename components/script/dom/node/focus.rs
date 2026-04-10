/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script_bindings::codegen::GenericBindings::ShadowRootBinding::ShadowRootMethods;
use script_bindings::inheritance::Castable;
use script_bindings::root::DomRoot;
use script_bindings::script_runtime::CanGc;

use crate::dom::document::focus::{
    FocusInitiator, FocusOperation, FocusableArea, FocusableAreaKind,
};
use crate::dom::types::{Element, HTMLDialogElement};
use crate::dom::{Node, NodeTraits, ShadowIncluding};

impl Node {
    /// Returns the appropriate [`FocusableArea`] when this [`Node`] is clicked on according to
    /// <https://www.w3.org/TR/pointerevents4/#handle-native-mouse-down>.
    ///
    /// Note that this is doing more than the specification which says to only take into account
    /// the node from the hit test. This isn't exactly how browsers work though, as they seem
    /// to look for the first inclusive ancestor node that has a focusable area associated with it.
    /// Note also that this may return [`FocusableArea::Viewport`].
    pub(crate) fn find_click_focusable_area(&self) -> FocusableArea {
        self.inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(|node| {
                node.get_the_focusable_area().filter(|focusable_area| {
                    focusable_area.kind().contains(FocusableAreaKind::Click)
                })
            })
            .unwrap_or(FocusableArea::Viewport)
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-focusable-area>
    ///
    /// There seems to be hole in the specification here. It describes how to get the focusable
    /// area for a focus target that isn't a focuable area, but is ambiguous about how to do
    /// this for a focus target that actually *is* a focusable area. The obvious thing is to
    /// just return the focus target, but it's still odd that this isn't mentioned in the
    /// specification.
    pub(crate) fn get_the_focusable_area(&self) -> Option<FocusableArea> {
        let kind = self
            .downcast::<Element>()
            .map(Element::focusable_area_kind)
            .unwrap_or_default();
        if !kind.is_empty() {
            return Some(FocusableArea::Node {
                node: DomRoot::from_ref(self),
                kind,
            });
        }
        self.get_the_focusable_area_if_not_a_focusable_area()
    }

    /// <https://html.spec.whatwg.org/multipage/#get-the-focusable-area>
    ///
    /// In addition to returning the DOM anchor of the focusable area for this [`Node`], this
    /// method also returns the [`FocusableAreaKind`] for efficiency reasons. Note that `None`
    /// is returned if this [`Node`] does not have a focusable area or if its focusable area
    /// is the `Document`'s viewport.
    ///
    /// TODO: It might be better to distinguish these two cases in the future.
    fn get_the_focusable_area_if_not_a_focusable_area(&self) -> Option<FocusableArea> {
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
        if self == self.owner_document().upcast::<Node>() {
            return Some(FocusableArea::Viewport);
        }

        // > ↪ If focus target is a navigable
        // >     Return the navigable's active document.
        // TODO: Implement this.

        // > ↪ If focus target is a navigable container with a non-null content navigable
        // >     Return the navigable container's content navigable's active document.
        // TODO: Implement this.

        // > ↪ If focus target is a shadow host whose shadow root's delegates focus is true
        if self
            .downcast::<Element>()
            .and_then(Element::shadow_root)
            .is_some_and(|shadow_root| shadow_root.DelegatesFocus())
        {
            // >   Step 1. Let focusedElement be the currently focused area of a top-level
            // >           traversable's DOM anchor.
            //
            // Note: This is a bit of a misnomer, because it might be a Node and not an Element.
            let document = self.owner_document();
            let focused_area = document.focus_handler().focused_area();
            let focused_element = focused_area.dom_anchor(&document);

            // >   Step 2. If focus target is a shadow-including inclusive ancestor of
            // >           focusedElement, then return focusedElement.
            if self
                .upcast::<Node>()
                .is_shadow_including_inclusive_ancestor_of(&focused_element)
            {
                return Some(focused_area.clone());
            }

            // >   Step 3. Return the focus delegate for focus target given focus trigger.
            return self.focus_delegate();
        }

        None
    }

    /// <https://html.spec.whatwg.org/multipage/#focus-delegate>
    ///
    /// In addition to returning the focus delegate for this [`Node`], this method also returns
    /// the [`FocusableAreaKind`] for efficiency reasons.
    fn focus_delegate(&self) -> Option<FocusableArea> {
        // > 1. If focusTarget is a shadow host and its shadow root's delegates focus is false, then
        // >    return null.
        let shadow_root = self.downcast::<Element>().and_then(Element::shadow_root);
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

            // > 6.2. If focusTarget is a dialog element and descendant is sequentially focusable, then
            // >      set focusableArea to descendant.
            let kind = descendant
                .downcast::<Element>()
                .map(Element::focusable_area_kind)
                .unwrap_or_default();
            if is_dialog_element && kind.contains(FocusableAreaKind::Sequential) {
                return Some(FocusableArea::Node {
                    node: descendant,
                    kind,
                });
            }

            // > 6.3. Otherwise, if focusTarget is not a dialog and descendant is a focusable area, set
            // >      focusableArea to descendant.
            if !kind.is_empty() {
                return Some(FocusableArea::Node {
                    node: descendant,
                    kind,
                });
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
    ///
    /// Return `true` if anything was focused or `false` otherwise.
    pub(crate) fn run_the_focusing_steps(
        &self,
        fallback_target: Option<FocusableArea>,
        can_gc: CanGc,
    ) -> bool {
        // > 1. If new focus target is not a focusable area, then set new focus target to the result
        // >    of getting the focusable area for new focus target, given focus trigger if it was
        // >    passed.
        // > 2. If new focus target is null, then:
        // > 2.1 If no fallback target was specified, then return.
        // > 2.2 Otherwise, set new focus target to the fallback target.
        let Some(focusable_area) = self.get_the_focusable_area().or(fallback_target) else {
            return false;
        };

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
        document.focus_handler().focus(
            FocusOperation::Focus(focusable_area),
            FocusInitiator::Local,
            can_gc,
        );
        true
    }
}
