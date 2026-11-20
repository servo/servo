/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use embedder_traits::EmbedderMsg;
use html5ever::local_name;
use js::context::{JSContext, NoGC};
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::dom::UnrootedDom;
use style::Atom;
use stylo_dom::ElementState;

use crate::dom::RootedPromise;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::document::document::{Document, LocalDescendantNavigablesIterator};
use crate::dom::element::element::Element;
use crate::dom::event::event::{EventBubbles, EventCancelable, EventComposed};
use crate::dom::event::eventtarget::EventTarget;
use crate::dom::html::embedded_content::htmliframeelement::HTMLIFrameElement;
use crate::dom::html::htmlelement::{HTMLElement, PopoverState};
use crate::dom::html::interactive::htmldialogelement::HTMLDialogElement;
use crate::dom::node::NodeTraits;
use crate::dom::promise::Promise;

/// The [`DocumentOverlayHandler`] is a structure responsible for overlays for
/// the [`crate::Document`] and storing data related to overlays. It exists to
/// decrease the size of the [`crate::Document`] structure.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentOverlayHandler {
    /// The [`Document`] this [`DocumentOverlayHandler`] belongs to.
    document: Dom<Document>,
    /// <https://fullscreen.spec.whatwg.org/#list-of-pending-fullscreen-events>
    list_of_pending_fullscreen_events: DomRefCell<Vec<PendingFullscreenEvent>>,
    /// <https://drafts.csswg.org/css-position-4/#document-top-layer>
    top_layer: DomRefCell<Vec<Dom<Element>>>,
    /// <https://drafts.csswg.org/css-position-4/#document-pending-top-layer-removals>
    pending_top_layer_removals: DomRefCell<Vec<Dom<Element>>>,
    /// <https://html.spec.whatwg.org/multipage/#blocked-by-a-modal-dialog>
    topmost_modal: MutNullableDom<Element>,
    /// <https://html.spec.whatwg.org/multipage/#open-dialogs-list>
    open_dialogs_list: DomRefCell<Vec<Dom<HTMLDialogElement>>>,
    /// <https://html.spec.whatwg.org/multipage/#dialog-pointerdown-target>
    dialog_pointerdown_target: MutNullableDom<HTMLDialogElement>,
    /// <https://html.spec.whatwg.org/multipage/#popover-pointerdown-target>
    popover_pointerdown_target: MutNullableDom<HTMLElement>,
    /// <https://html.spec.whatwg.org/multipage/#hiding-popover-nesting-count>
    hiding_popover_nesting_count: Cell<u32>,
    /// <https://html.spec.whatwg.org/multipage/#showing-popover>
    showing_popover: Cell<bool>,
    /// <https://html.spec.whatwg.org/multipage/#hint-stack-parent>
    hint_stack_parent: MutNullableDom<HTMLElement>,
}

impl DocumentOverlayHandler {
    pub(crate) fn new(document: &Document) -> Self {
        Self {
            document: Dom::from_ref(document),
            list_of_pending_fullscreen_events: Default::default(),
            top_layer: Default::default(),
            pending_top_layer_removals: Default::default(),
            topmost_modal: Default::default(),
            open_dialogs_list: Default::default(),
            dialog_pointerdown_target: Default::default(),
            popover_pointerdown_target: Default::default(),
            hiding_popover_nesting_count: Default::default(),
            showing_popover: Default::default(),
            hint_stack_parent: Default::default(),
        }
    }

    /// <https://fullscreen.spec.whatwg.org/#fullscreen-element>
    pub(crate) fn fullscreen_element(&self) -> Option<DomRoot<Element>> {
        // > The fullscreen element is the topmost element in the document’s top layer whose
        // > fullscreen flag is set, if any, and null otherwise.
        self.top_layer
            .borrow()
            .iter()
            .rfind(|element| element.fullscreen_flag())
            .map(|element| element.as_rooted())
    }

    pub(crate) fn append_to_list_of_pending_fullscreen_events(
        &self,
        no_gc: &NoGC,
        event_type: Atom,
        element: &Element,
    ) {
        self.list_of_pending_fullscreen_events
            .safe_borrow_mut(no_gc)
            .push(PendingFullscreenEvent {
                event_type,
                element: Dom::from_ref(element),
            });
    }

    pub(crate) fn has_pending_fullscreen_events(&self) -> bool {
        !self.list_of_pending_fullscreen_events.borrow().is_empty()
    }

    /// <https://fullscreen.spec.whatwg.org/#fullscreen-an-element>
    pub(crate) fn fullscreen_an_element(&self, cx: &mut JSContext, element: &Element) {
        // Step 1. Let hideUntil be the result of running topmost popover ancestor given element,
        // null, and false.
        let hide_until = element.topmost_popover_ancestor(cx.no_gc(), None, false);

        // Step 2. Run hide popovers until given element’s node document, hideUntil, false, and
        // true.
        element
            .owner_document()
            .overlay_handler()
            .hide_popovers_until(cx, hide_until.as_deref(), false, true);

        // Step 3. Set element’s fullscreen flag.
        element.set_fullscreen_state(cx.no_gc(), true);

        // Step 4. Remove from the top layer immediately given element.
        self.remove_an_element_from_the_top_layer_immediately(element, cx.no_gc());

        // Step 5. Add to the top layer given element.
        self.add_an_element_to_the_top_layer(element, cx.no_gc());
    }

    /// <https://fullscreen.spec.whatwg.org/#unfullscreen-an-element>
    pub(crate) fn unfullscreen_an_element(&self, no_gc: &NoGC, element: &Element) {
        // > To unfullscreen an element, unset element’s fullscreen flag, iframe fullscreen flag
        // > (if any), and keyboard lock flag, and remove from the top layer immediately given
        // > element.
        element.set_fullscreen_state(no_gc, false);
        if let Some(iframe) = element.downcast::<HTMLIFrameElement>() {
            iframe.set_iframe_fullscreen_flag(false);
        }
        // FIXME: Unset element’s keyboard lock flag.
        self.remove_an_element_from_the_top_layer_immediately(element, no_gc);
    }

    /// <https://fullscreen.spec.whatwg.org/#unfullscreen-a-document>
    pub(crate) fn unfullscreen_a_document(&self, no_gc: &NoGC) {
        // > To unfullscreen a document, unfullscreen all elements, within document’s top layer,
        // > whose fullscreen flag is set.
        let fullscreened_elements = self
            .top_layer
            .borrow()
            .iter()
            .filter(|element| element.fullscreen_flag())
            .map(|element| element.as_unrooted(no_gc))
            .collect::<Vec<_>>();
        for element in fullscreened_elements {
            self.unfullscreen_an_element(no_gc, &element);
        }
    }

    /// <https://fullscreen.spec.whatwg.org/#fully-exit-fullscreen>
    pub(crate) fn fully_exit_fullscreen(&self, cx: &mut JSContext) {
        // Step 1. If document’s fullscreen element is null, terminate these steps.
        let Some(fullscreen_element) = self.fullscreen_element() else {
            return;
        };

        // Step 2. Unfullscreen elements whose fullscreen flag is set, within document’s top
        // layer, except for document’s fullscreen element.
        let elements = self
            .top_layer
            .borrow()
            .iter()
            .filter(|element| element.fullscreen_flag() && ***element != *fullscreen_element)
            .map(|element| element.as_unrooted(cx.no_gc()))
            .collect::<Vec<_>>();
        for element in elements {
            self.unfullscreen_an_element(cx.no_gc(), &element);
        }

        // Step 3. Exit fullscreen document.
        let _ = self.exit_fullscreen(cx);
    }

    /// <https://fullscreen.spec.whatwg.org/#fullscreen-is-supported>
    pub(crate) fn fullscreen_is_supported(&self) -> bool {
        // > Fullscreen is supported if there is no previously-established user preference,
        // > security risk, or platform limitation.
        true
    }

    /// <https://fullscreen.spec.whatwg.org/#run-the-fullscreen-steps>
    pub(crate) fn run_the_fullscreen_steps(&self, cx: &mut JSContext) {
        // Step 1. Let pendingEvents be a clone of document’s list of pending fullscreen events.
        // Step 2. Empty document’s list of pending fullscreen events.
        let pending_events: Vec<(Atom, DomRoot<Element>)> = self
            .list_of_pending_fullscreen_events
            .safe_borrow_mut(cx.no_gc())
            .drain(..)
            .map(|event| (event.event_type.clone(), event.element.as_rooted()))
            .collect();

        // Step 3. For each (type, element) in pendingEvents:
        for (event_type, element) in pending_events {
            // Step 3.1. Let target be element if element is connected and its node document is
            // document, and otherwise let target be document.
            let target: DomRoot<EventTarget> =
                if element.is_connected() && *element.owner_document() == *self.document {
                    DomRoot::upcast(element)
                } else {
                    DomRoot::upcast(self.document.as_rooted())
                };

            // Step 3.2. Fire an event named type, with its bubbles and composed attributes set to
            // true, at target.
            target.fire_event_with_params(
                cx,
                event_type,
                EventBubbles::Bubbles,
                EventCancelable::NotCancelable,
                EventComposed::Composed,
            );
        }
    }

    /// <https://fullscreen.spec.whatwg.org/#simple-fullscreen-document>
    pub(crate) fn is_simple_fullscreen_document(&self) -> bool {
        // > A document is said to be a simple fullscreen document if there is exactly one element
        // > in its top layer that has its fullscreen flag set.
        self.top_layer
            .borrow()
            .iter()
            .filter(|element| element.fullscreen_flag())
            .count() ==
            1
    }

    /// <https://fullscreen.spec.whatwg.org/#collect-documents-to-unfullscreen>
    pub(crate) fn collect_documents_to_unfullscreen(&self) -> Vec<DomRoot<Document>> {
        // Step 1. Let docs be an ordered set consisting of doc.
        let mut docs = vec![self.document.as_rooted()];

        // Step 2. While true:
        loop {
            // Step 2.1. Let lastDoc be docs’s last document.
            let last_doc = docs
                .last()
                .expect("docs is initialized with an element")
                .clone();

            // Step 2.2. Assert: lastDoc’s fullscreen element is not null.
            assert!(last_doc.overlay_handler().fullscreen_element().is_some());

            // Step 2.3. If lastDoc is not a simple fullscreen document, break.
            if !last_doc.overlay_handler().is_simple_fullscreen_document() {
                break;
            }

            // Step 2.4. Let container be lastDoc’s node navigable’s container.
            // Step 2.5. If container is null, then break.
            // FIXME: Support cross-origin container
            let window_proxy = last_doc.window().window_proxy();
            let Some(container) = window_proxy.frame_element() else {
                break;
            };

            // Step 2.6. If container’s iframe fullscreen flag is set, break.
            if container
                .downcast::<HTMLIFrameElement>()
                .is_some_and(|iframe| iframe.iframe_fullscreen_flag())
            {
                break;
            }

            // Step 2.7. Append container’s node document to docs.
            docs.push(container.owner_document());
        }

        // Step 3. Return docs.
        docs
    }

    /// <https://fullscreen.spec.whatwg.org/#exit-fullscreen>
    pub(crate) fn exit_fullscreen(&self, cx: &mut JSContext) -> RootedPromise {
        // Step 1. Let promise be a new promise.
        let mut realm = CurrentRealm::assert(cx);
        let promise = Promise::new_in_realm(&mut realm);

        // Step 2. If doc is not fully active or doc’s fullscreen element is null, then reject
        // promise with a TypeError exception and return promise.
        let mut doc = self.document.as_rooted();
        if !doc.is_fully_active() || self.fullscreen_element().is_none() {
            promise.reject_error(
                cx,
                Error::Type(
                    c"No fullscreen element to exit or document is not fully active".to_owned(),
                ),
            );
            return promise;
        }

        // Step 3. Let resize be false.
        let mut resize = false;

        // Step 4. Let docs be the result of collecting documents to unfullscreen given doc.
        let docs = self.collect_documents_to_unfullscreen();

        // Step 5. Let topLevelDoc be doc’s node navigable’s top-level traversable’s active
        // document.
        // FIXME: Support cross-origin document
        let top_level_doc = doc.window().top_level_document_if_local();

        // Step 6. If topLevelDoc is in docs, and it is a simple fullscreen document, then set doc
        // to topLevelDoc and resize to true.
        if let Some(top_level_doc) = top_level_doc &&
            docs.contains(&top_level_doc) &&
            top_level_doc
                .overlay_handler()
                .is_simple_fullscreen_document()
        {
            doc = top_level_doc;
            resize = true;
        }

        // Step 7. If doc’s fullscreen element is not connected:
        if let Some(fullscreen_element) = doc.overlay_handler().fullscreen_element() &&
            !fullscreen_element.is_connected()
        {
            // Step 7.1. Append (fullscreenchange, doc’s fullscreen element) to doc’s list of
            // pending fullscreen events.
            doc.overlay_handler()
                .append_to_list_of_pending_fullscreen_events(
                    cx.no_gc(),
                    atom!("fullscreenchange"),
                    &fullscreen_element,
                );

            // Step 7.2. Unfullscreen doc’s fullscreen element.
            doc.overlay_handler()
                .unfullscreen_an_element(cx.no_gc(), &fullscreen_element);
        }

        // Step 8. Return promise, and run the remaining steps in parallel.
        let trusted_doc = Trusted::new(&*doc);
        let trusted_promise = TrustedPromise::from(&promise);
        doc.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue(task!(exit_fullscreen_steps: move |cx| {
                let doc = trusted_doc.root();
                let promise = trusted_promise.root(cx);

                // FIXME: Step 9. Run the fully unlock the screen orientation steps with doc.

                // Step 10. If resize is true, resize doc’s viewport to its "normal" dimensions.
                if resize {
                    doc.send_to_embedder(EmbedderMsg::NotifyFullscreenStateChanged(
                        doc.webview_id(),
                        false,
                    ));
                }

                // Step 11. If doc’s fullscreen element is null, then resolve promise with undefined
                // and terminate these steps.
                if doc.overlay_handler().fullscreen_element().is_none() {
                    promise.resolve_native(cx, &());
                    return;
                }

                // Step 12. Let exitDocs be the result of collecting documents to unfullscreen given
                // doc.
                let exit_docs = doc.overlay_handler().collect_documents_to_unfullscreen();

                // Step 13. Let descendantDocs be an ordered set consisting of doc’s descendant
                // navigables' active documents whose fullscreen element is non-null, if any, in
                // tree order.
                let descendant_docs = LocalDescendantNavigablesIterator::new(&doc)
                    .filter_map(|descendant_doc| {
                        let element = descendant_doc.overlay_handler().fullscreen_element()?;
                        Some((descendant_doc, element))
                    })
                    .collect::<Vec<_>>();

                // Step 14. For each exitDoc in exitDocs:
                for exit_doc in exit_docs {
                    let fullscreen_element = exit_doc.overlay_handler().fullscreen_element().expect("The document is asserted to have a fullscreen element in collect_documents_to_unfullscreen");

                    // Step 14.1. Append (fullscreenchange, exitDoc’s fullscreen element) to
                    // exitDoc’s list of pending fullscreen events.
                    exit_doc
                        .overlay_handler()
                        .append_to_list_of_pending_fullscreen_events(
                            cx.no_gc(),
                            atom!("fullscreenchange"),
                            &fullscreen_element,
                        );

                    // Step 14.2. If resize is true, unfullscreen exitDoc.
                    if resize {
                        exit_doc.overlay_handler().unfullscreen_a_document(cx.no_gc());
                    }
                    // Step 14.3. Otherwise, unfullscreen exitDoc’s fullscreen element.
                    else {
                        exit_doc
                            .overlay_handler()
                            .unfullscreen_an_element(cx.no_gc(), &fullscreen_element);
                    }
                }

                // Step 15. For each descendantDoc in descendantDocs:
                for (descendant_doc, fullscreen_element) in descendant_docs {
                    // Step 15.1. Append (fullscreenchange, descendantDoc’s fullscreen element) to
                    // descendantDoc’s list of pending fullscreen events.
                    descendant_doc
                        .overlay_handler()
                        .append_to_list_of_pending_fullscreen_events(
                            cx.no_gc(),
                            atom!("fullscreenchange"),
                            &fullscreen_element,
                        );

                    // Step 15.2. Unfullscreen descendantDoc.
                    descendant_doc
                        .overlay_handler()
                        .unfullscreen_a_document(cx.no_gc());
                }

                // Step 16. Resolve promise with undefined.
                promise.resolve_native(cx, &());
            }));

        promise
    }

    /// <https://html.spec.whatwg.org/multipage/#allowed-to-use>
    pub(crate) fn is_allowed_to_use_fullscreen(&self) -> bool {
        // Step 1. If document's browsing context is null, then return false.
        if self.document.browsing_context().is_none() {
            return false;
        };

        // Step 2. If document is not fully active, then return false.
        if !self.document.is_fully_active() {
            return false;
        }

        // 3. If the result of running is feature enabled in document for origin on feature,
        // document, and document's origin is "Enabled", then return true.
        //
        // FIXME: Properly check if feature is enabled, for now we do a makeshift attribute check
        // for iFrames and otherwise assume it always is.
        if self.document.window().is_top_level() {
            true
        } else {
            self.document
                .window()
                .GetFrameElement()
                .is_some_and(|el| el.has_attribute(&local_name!("allowfullscreen")))
        }

        // FIXME: Return false.
    }

    pub(crate) fn open_dialogs_list_contains(&self, dialog: &HTMLDialogElement) -> bool {
        self.open_dialogs_list
            .borrow()
            .iter()
            .any(|open_dialog| &**open_dialog == dialog)
    }

    pub(crate) fn add_to_open_dialogs_list(&self, dialog: &HTMLDialogElement, no_gc: &NoGC) {
        self.open_dialogs_list
            .safe_borrow_mut(no_gc)
            .push(Dom::from_ref(dialog));
    }

    pub(crate) fn remove_from_open_dialogs_list(&self, dialog: &HTMLDialogElement, no_gc: &NoGC) {
        self.open_dialogs_list
            .safe_borrow_mut(no_gc)
            .retain(|open| &**open != dialog);
    }

    pub(crate) fn topmost_open_dialog(&self) -> Option<DomRoot<HTMLDialogElement>> {
        self.open_dialogs_list
            .borrow()
            .last()
            .map(|dialog| dialog.as_rooted())
    }

    pub(crate) fn dialog_pointerdown_target(&self) -> Option<DomRoot<HTMLDialogElement>> {
        self.dialog_pointerdown_target.get()
    }

    pub(crate) fn set_dialog_pointerdown_target(&self, dialog: Option<&HTMLDialogElement>) {
        self.dialog_pointerdown_target.set(dialog);
    }

    pub(crate) fn popover_pointerdown_target(&self) -> Option<DomRoot<HTMLElement>> {
        self.popover_pointerdown_target.get()
    }

    pub(crate) fn set_popover_pointerdown_target(&self, element: Option<&HTMLElement>) {
        self.popover_pointerdown_target.set(element);
    }

    pub(crate) fn top_layer_contains(&self, element: &Element) -> bool {
        self.top_layer
            .borrow()
            .iter()
            .any(|top_layer_element| &**top_layer_element == element)
    }

    fn pending_top_layer_removals_contains(&self, element: &Element) -> bool {
        self.pending_top_layer_removals
            .borrow()
            .iter()
            .any(|pending_element| &**pending_element == element)
    }

    /// <https://drafts.csswg.org/css-position-4/#in-the-top-layer>
    pub(crate) fn is_in_the_top_layer(&self, element: &Element) -> bool {
        // > An element el is in the top layer if el is contained in its node document’s top layer
        // > but not contained in its node document’s pending top layer removals.
        self.top_layer_contains(element) && !self.pending_top_layer_removals_contains(element)
    }

    /// <https://drafts.csswg.org/css-position-4/#add-an-element-to-the-top-layer>
    pub(crate) fn add_an_element_to_the_top_layer(&self, element: &Element, no_gc: &NoGC) {
        // Step 1. Let doc be el’s node document.

        // Step 2. If el is already contained in doc's top layer:
        if self.top_layer_contains(element) {
            // Step 2.1. Assert: el is also in doc's pending top layer removals. (Otherwise, this is
            // a spec error.)
            assert!(self.pending_top_layer_removals_contains(element));

            // Step 2.2. Remove el from both doc's top layer and pending top layer removals.
            self.top_layer
                .safe_borrow_mut(no_gc)
                .retain(|top_layer_element| &**top_layer_element != element);
            self.pending_top_layer_removals
                .safe_borrow_mut(no_gc)
                .retain(|pending_element| &**pending_element != element);
        }

        // Step 3. Append el to doc's top layer.
        self.top_layer
            .safe_borrow_mut(no_gc)
            .push(Dom::from_ref(element));

        // TODO: Step 4. At the UA !important cascade origin, add a rule targeting el
        // containing an overlay: auto declaration.

        self.update_topmost_modal(no_gc);
    }

    /// <https://drafts.csswg.org/css-position-4/#request-an-element-to-be-removed-from-the-top-layer>
    pub(crate) fn request_an_element_to_be_removed_from_the_top_layer(
        &self,
        element: &Element,
        no_gc: &NoGC,
    ) {
        // Step 1. Let doc be el's node document.

        // Step 2. If el is not contained doc's top layer, or el is already contained in doc's
        // pending top layer removals, return.
        if !self.top_layer_contains(element) || self.pending_top_layer_removals_contains(element) {
            return;
        }

        // TODO: Step 3. Remove the UA !important overlay: auto rule targeting el.

        // Step 4. Append el to doc's pending top layer removals.
        self.pending_top_layer_removals
            .safe_borrow_mut(no_gc)
            .push(Dom::from_ref(element));

        self.update_topmost_modal(no_gc);
    }

    /// <https://drafts.csswg.org/css-position-4/#remove-an-element-from-the-top-layer-immediately>
    pub(crate) fn remove_an_element_from_the_top_layer_immediately(
        &self,
        element: &Element,
        no_gc: &NoGC,
    ) {
        // Step 1. Let doc be el's node document.

        // Step 2. Remove el from doc's top layer and pending top layer removals.
        self.top_layer
            .safe_borrow_mut(no_gc)
            .retain(|top_layer_element| &**top_layer_element != element);
        self.pending_top_layer_removals
            .safe_borrow_mut(no_gc)
            .retain(|pending_element| &**pending_element != element);

        // TODO: Step 3. Remove the UA !important overlay: auto rule targeting el, if it exists.

        self.update_topmost_modal(no_gc);
    }

    /// <https://drafts.csswg.org/css-position-4/#process-top-layer-removals>
    pub(crate) fn process_top_layer_removals(&self, cx: &mut JSContext) {
        // Step 1. For each element el in doc’s pending top layer removals: if el's computed value
        // of overlay is none, or el is not rendered, remove from the top layer immediately el.
        // TODO: Check `overlay` once it is supported.
        let pending_removals: Vec<_> = self
            .pending_top_layer_removals
            .borrow()
            .iter()
            .map(|element| DomRoot::from_ref(&**element))
            .collect();
        for element in pending_removals {
            if !element.has_css_layout_box_deprecated(cx) {
                self.remove_an_element_from_the_top_layer_immediately(&element, cx)
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#blocked-by-a-modal-dialog>
    pub fn update_topmost_modal(&self, no_gc: &NoGC) {
        // > A Document document is blocked by a modal dialog subject if subject is the topmost
        // > dialog element in document's top layer.
        let new_topmost_modal = self
            .top_layer
            .borrow()
            .iter()
            .rev()
            .find(|element| {
                element.state().contains(ElementState::MODAL) &&
                    !self.pending_top_layer_removals_contains(element)
            })
            .map(|element| DomRoot::from_ref(&**element));
        let old_topmost_modal = self.topmost_modal.get();

        if let Some(old_topmost_modal) = old_topmost_modal {
            old_topmost_modal.set_state(no_gc, ElementState::TOPMOST_MODAL, false);
        }
        if let Some(new_topmost_modal) = &new_topmost_modal {
            new_topmost_modal.set_state(no_gc, ElementState::TOPMOST_MODAL, true);
        }

        self.topmost_modal.set(new_topmost_modal.as_deref());

        // > While document is so blocked, every node that is connected to document, with the
        // > exception of the subject element and its flat tree descendants, must become inert.
        if let Some(document_element) = self.document.GetDocumentElement() {
            let is_inert = new_topmost_modal.is_some() ||
                document_element.has_attribute(&local_name!("inert"));
            document_element.set_state(no_gc, ElementState::INERT, is_inert);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#blocked-by-a-modal-dialog>
    pub(crate) fn is_blocked_by_a_modal_dialog(&self) -> bool {
        self.topmost_modal.get().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#hide-popovers-until>
    pub(crate) fn hide_popovers_until(
        &self,
        cx: &mut JSContext,
        endpoint: Option<&HTMLElement>,
        focus_previous_element: bool,
        fire_events: bool,
    ) {
        // Step 1. Let endpointIsHint be true if document's showing hint popover list contains
        // endpoint; otherwise false.
        let endpoint_is_hint = endpoint.is_some_and(|endpoint| {
            self.showing_popover_list_contains(cx.no_gc(), PopoverState::Hint, endpoint)
        });

        // Step 2. Run hide popover stack until given document, endpoint, Hint,
        // focusPreviousElement, and fireEvents.
        self.hide_popover_stack_until(
            cx,
            endpoint,
            PopoverState::Hint,
            focus_previous_element,
            fire_events,
        );

        // Step 3. Let autoEndpoint be endpoint.
        //
        // Need to keep the reference to the hint_parent alive such that we
        // can dereference to obtain its endpoint
        let hint_parent;
        let mut auto_endpoint = endpoint;

        // Step 4. If endpointIsHint is true, then set autoEndpoint to document's hint stack
        // parent.
        if endpoint_is_hint {
            hint_parent = self.hint_stack_parent.get();
            auto_endpoint = hint_parent.as_deref();
        }

        // Step 5. Run hide popover stack until given document, autoEndpoint, Auto,
        // focusPreviousElement, and fireEvents.
        self.hide_popover_stack_until(
            cx,
            auto_endpoint,
            PopoverState::Auto,
            focus_previous_element,
            fire_events,
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#hide-popover-stack-until>
    pub(crate) fn hide_popover_stack_until(
        &self,
        cx: &mut JSContext,
        endpoint: Option<&HTMLElement>,
        stack_type: PopoverState,
        focus_previous_element: bool,
        fire_events: bool,
    ) {
        // Step 1. Let popoverList be document's showing auto popover list if stackType is Auto,
        // and document's showing hint popover list otherwise.
        let popover_list: Vec<DomRoot<HTMLElement>> = self
            .showing_popover_list_unrooted(cx.no_gc(), stack_type)
            .iter()
            .map(|popover| popover.as_rooted())
            .collect();

        // Step 2. Let lastHideIndex be 0 if popoverList does not contain endpoint; otherwise the
        // index of endpoint in popoverList plus 1.
        let last_hide_index = endpoint
            .and_then(|endpoint| {
                popover_list
                    .iter()
                    .position(|popover| &**popover == endpoint)
            })
            .map_or(0, |index| index + 1);

        // Step 3. Let toHide be popoverList from lastHideIndex onwards, in reverse order.
        let to_hide = popover_list[last_hide_index..].iter().rev();

        // Step 4. Let toRemain be popoverList up to lastHideIndex.
        let to_remain = &popover_list[..last_hide_index];

        // Step 5. For each popover of toHide, run the hide popover algorithm given popover,
        // focusPreviousElement, fireEvents, false, and null.
        for popover in to_hide {
            let _ = popover.hide_popover(cx, focus_previous_element, fire_events, false, None);
        }

        // Step 6. Let newPopoverList be document's showing auto popover list if stackType is
        // Auto, and document's showing hint popover list otherwise.
        let new_popover_list: Vec<DomRoot<HTMLElement>> = self
            .showing_popover_list_unrooted(cx.no_gc(), stack_type)
            .iter()
            .map(|popover| popover.as_rooted())
            .collect();

        // Step 7. Let toCheck be newPopoverList in reverse order.
        let to_check = new_popover_list.iter().rev();

        // Step 8. For each popover of toCheck:
        for popover in to_check {
            // Step 8.1. If toRemain contains popover, then continue.
            if to_remain.contains(popover) {
                continue;
            }

            // Step 8.2. Run the hide popover algorithm given popover, focusPreviousElement, false,
            // false, and null.
            let _ = popover.hide_popover(cx, focus_previous_element, false, false, None);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#topmost-auto-or-hint-popover>
    pub(crate) fn find_the_topmost_auto_or_hint_popover(
        &self,
        no_gc: &NoGC,
    ) -> Option<DomRoot<HTMLElement>> {
        // Step 1. If document's showing hint popover list is not empty, then return its last
        // element.
        // Step 2. If document's showing auto popover list is not empty, then return its last
        // element.
        // Step 3. Return null.
        //
        // We use pop instead of last to take ownership of its last element. The Vec is temporary,
        // so mutating it is fine.
        self.showing_popover_list_unrooted(no_gc, PopoverState::Hint)
            .pop()
            .or_else(|| {
                self.showing_popover_list_unrooted(no_gc, PopoverState::Auto)
                    .pop()
            })
            .map(|popover| popover.as_rooted())
    }

    /// <https://html.spec.whatwg.org/multipage/#auto-popover-list>
    /// <https://html.spec.whatwg.org/multipage/#showing-hint-popover-list>
    ///
    /// The spec lists these steps twice, once with "auto" and once with "hint" as the mode in step 2.1.
    /// `mode` must be [`PopoverState::Auto`] or [`PopoverState::Hint`].
    pub(crate) fn showing_popover_list_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
        mode: PopoverState,
    ) -> Vec<UnrootedDom<'a, HTMLElement>> {
        debug_assert!(matches!(mode, PopoverState::Auto | PopoverState::Hint));

        // Step 1. Let popovers be « ».
        // Step 2. For each Element element of document's top layer:
        // Step 2.1. If all of the following are true:
        // - element is an HTML element;
        // - element's opened in popover mode is mode; and
        // - element's popover visibility state is showing,
        // - then append element to popovers.
        // Step 3. Return popovers.
        self.top_layer
            .borrow()
            .iter()
            .filter_map(|element| element.downcast::<HTMLElement>())
            .filter(|element| {
                element
                    .upcast::<Element>()
                    .popover_visibility_state_showing() &&
                    element.opened_in_popover_mode() == Some(mode)
            })
            .map(|element| UnrootedDom::from_ref(element, no_gc))
            .collect()
    }

    pub(crate) fn showing_popover_list_contains(
        &self,
        no_gc: &NoGC,
        mode: PopoverState,
        element: &HTMLElement,
    ) -> bool {
        self.showing_popover_list_unrooted(no_gc, mode)
            .iter()
            .any(|popover| ***popover == *element)
    }

    pub(crate) fn showing_popover(&self) -> bool {
        self.showing_popover.get()
    }

    pub(crate) fn set_showing_popover(&self, value: bool) {
        self.showing_popover.set(value);
    }

    pub(crate) fn hiding_popover_nesting_count(&self) -> u32 {
        self.hiding_popover_nesting_count.get()
    }

    pub(crate) fn increment_hiding_popover_nesting_count(&self) {
        self.hiding_popover_nesting_count
            .set(self.hiding_popover_nesting_count.get() + 1);
    }

    pub(crate) fn decrement_hiding_popover_nesting_count(&self) {
        self.hiding_popover_nesting_count
            .set(self.hiding_popover_nesting_count.get() - 1);
    }

    pub(crate) fn is_hint_stack_parent(&self, element: &HTMLElement) -> bool {
        self.hint_stack_parent
            .get()
            .is_some_and(|parent| &*parent == element)
    }

    pub(crate) fn set_hint_stack_parent(&self, element: Option<&HTMLElement>) {
        self.hint_stack_parent.set(element);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
struct PendingFullscreenEvent {
    #[no_trace]
    event_type: Atom,
    element: Dom<Element>,
}
