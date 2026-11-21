/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use embedder_traits::EmbedderMsg;
use html5ever::{local_name, ns};
use js::context::{JSContext, NoGC};
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use servo_config::pref;
use stylo_dom::ElementState;

use crate::dom::RootedPromise;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::document::document::Document;
use crate::dom::element::element::Element;
use crate::dom::event::event::{EventBubbles, EventCancelable, EventComposed};
use crate::dom::event::eventtarget::EventTarget;
use crate::dom::html::interactive::htmldialogelement::HTMLDialogElement;
use crate::dom::node::NodeTraits;
use crate::dom::promise::Promise;
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::runtime::script_runtime::ScriptThreadEventCategory;
use crate::tasks::task::TaskOnce;
use crate::tasks::task_source::TaskSourceName;

/// The [`DocumentOverlayHandler`] is a structure responsible for overlays for
/// the [`crate::Document`] and storing data related to overlays. It exists to
/// decrease the size of the [`crate::Document`] structure.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DocumentOverlayHandler {
    /// The [`Document`] this [`DocumentOverlayHandler`] belongs to.
    document: Dom<Document>,
    /// Entry node for fullscreen.
    fullscreen_element: MutNullableDom<Element>,
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
}

impl DocumentOverlayHandler {
    pub(crate) fn new(document: &Document) -> Self {
        Self {
            document: Dom::from_ref(document),
            fullscreen_element: MutNullableDom::new(None),
            top_layer: Default::default(),
            pending_top_layer_removals: Default::default(),
            topmost_modal: Default::default(),
            open_dialogs_list: Default::default(),
            dialog_pointerdown_target: Default::default(),
        }
    }

    pub(crate) fn fullscreen_element(&self) -> Option<DomRoot<Element>> {
        self.fullscreen_element.get()
    }

    pub(crate) fn set_fullscreen_element(&self, element: Option<&Element>) {
        self.fullscreen_element.set(element);
    }

    /// <https://fullscreen.spec.whatwg.org/#fully-exit-fullscreen>
    pub(crate) fn fully_exit_fullscreen(&self, cx: &mut JSContext) {
        // Step 1. If document’s fullscreen element is null, terminate these
        // steps.
        if self.fullscreen_element().is_none() {
            return;
        };

        // TODO Step 2. Unfullscreen elements whose fullscreen flag is set,
        // within document’s top layer, except for document’s fullscreen element.

        // Step 3. Exit fullscreen document.
        let _ = self.exit_fullscreen(cx);
    }

    /// <https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen>
    pub(crate) fn enter_fullscreen(
        &self,
        cx: &mut CurrentRealm,
        pending: &Element,
    ) -> RootedPromise {
        // Step 1
        // > Let pendingDoc be this’s node document.
        let document = pending.owner_document();

        // Step 2
        // > Let promise be a new promise.
        let promise = Promise::new_in_realm(cx);

        // Step 3
        // > If pendingDoc is not fully active, then reject promise with a TypeError exception and return promise.
        if !document.is_fully_active() {
            promise.reject_error(cx, Error::Type(c"Document is not fully active".to_owned()));
            return promise;
        }

        // Step 4
        // > Let error be false.
        let mut error = false;

        // Step 5
        // > If any of the following conditions are false, then set error to true:
        {
            // > - This’s namespace is the HTML namespace or this is an SVG svg or MathML math element. [SVG] [MATHML]
            match *pending.namespace() {
                ns!(mathml) => {
                    if *pending.local_name() != local_name!("math") {
                        error = true;
                    }
                },
                ns!(svg) => {
                    if *pending.local_name() != local_name!("svg") {
                        error = true;
                    }
                },
                ns!(html) => (),
                _ => error = true,
            }

            // > - This is not a dialog element.
            if pending.is::<HTMLDialogElement>() {
                error = true;
            }

            // > - The fullscreen element ready check for this returns true.
            if !pending.fullscreen_element_ready_check() {
                error = true;
            }

            // > - Fullscreen is supported.
            // <https://fullscreen.spec.whatwg.org/#fullscreen-is-supported>
            // > Fullscreen is supported if there is no previously-established user preference, security risk, or platform limitation.
            // TODO: Add checks for whether fullscreen is supported as definition.

            // > - This’s relevant global object has transient activation or the algorithm is triggered by a user generated orientation change.
            // TODO: implement screen orientation API
            if !pending.owner_window().has_transient_activation() {
                error = true;
            }
        }

        if pref!(dom_fullscreen_test) {
            // For reftests we just take over the current window,
            // and don't try to really enter fullscreen.
            info!("Tests don't really enter fullscreen.");
        } else {
            // TODO fullscreen is supported
            // TODO This algorithm is allowed to request fullscreen.
            warn!("Fullscreen not supported yet");
        }

        // Step 6
        // > If error is false, then consume user activation given pendingDoc’s relevant global object.
        if !error {
            pending
                .owner_window_unrooted(cx.no_gc())
                .consume_user_activation();
        }

        // Step 8.
        // > If error is false, then resize pendingDoc’s node navigable’s top-level traversable’s active document’s viewport’s dimensions,
        // > optionally taking into account options["navigationUI"]:
        // TODO(#21600): Improve spec compliance of steps 7-13 paralelism.
        // TODO(#42064): Implement fullscreen options, and ensure that this is spec compliant for all embedder.
        if !error {
            let event = EmbedderMsg::NotifyFullscreenStateChanged(document.webview_id(), true);
            document.send_to_embedder(event);
        }

        // Step 7
        // > Return promise, and run the remaining steps in parallel.
        let pipeline_id = self.document.window().pipeline_id();

        let trusted_pending = Trusted::new(pending);
        let trusted_pending_doc = Trusted::new(&*document);
        let trusted_promise = TrustedPromise::from(&promise);
        let handler = ElementPerformFullscreenEnter::new(
            trusted_pending,
            trusted_pending_doc,
            trusted_promise,
            error,
        );
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::EnterFullscreen,
            handler,
            Some(pipeline_id),
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        self.document
            .window()
            .main_thread_script_chan()
            .send(msg)
            .unwrap();

        promise
    }

    /// <https://fullscreen.spec.whatwg.org/#exit-fullscreen>
    pub(crate) fn exit_fullscreen(&self, cx: &mut JSContext) -> RootedPromise {
        let document = &*self.document;

        // Step 1
        // > Let promise be a new promise
        let mut realm = CurrentRealm::assert(cx);
        let promise = Promise::new_in_realm(&mut realm);

        // Step 2
        // > If doc is not fully active or doc’s fullscreen element is null, then reject promise with a TypeError exception and return promise.
        if !document.is_fully_active() || self.fullscreen_element().is_none() {
            promise.reject_error(
                cx,
                Error::Type(
                    c"No fullscreen element to exit or document is not fully active".to_owned(),
                ),
            );
            return promise;
        }

        // TODO(#42067): Implement step 3-7, handling fullscreen's propagation across navigables.

        let element = self.fullscreen_element().unwrap();
        // Step 10
        // > If resize is true, resize doc’s viewport to its "normal" dimensions.
        // TODO(#21600): Improve spec compliance of steps 8-15 paralelism.
        let event = EmbedderMsg::NotifyFullscreenStateChanged(document.webview_id(), false);
        document.send_to_embedder(event);

        // Step 8
        // > Return promise, and run the remaining steps in parallel.
        let trusted_element = Trusted::new(&*element);
        let trusted_promise = TrustedPromise::from(&promise);
        let handler = ElementPerformFullscreenExit::new(trusted_element, trusted_promise);
        let pipeline_id = Some(self.document.window().pipeline_id());
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::ExitFullscreen,
            handler,
            pipeline_id,
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        self.document
            .window()
            .main_thread_script_chan()
            .send(msg)
            .unwrap();

        promise
    }

    /// <https://html.spec.whatwg.org/multipage/#allowed-to-use>
    pub(crate) fn get_allow_fullscreen(&self) -> bool {
        match self.document.browsing_context() {
            // Step 1
            None => false,
            Some(_) => {
                // Step 2
                if self.document.window().is_top_level() {
                    true
                } else {
                    // Step 3
                    self.document
                        .window()
                        .GetFrameElement()
                        .is_some_and(|el| el.has_attribute(&local_name!("allowfullscreen")))
                }
            },
        }
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
            if !element.has_css_layout_box(cx) {
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
}

struct ElementPerformFullscreenEnter {
    element: Trusted<Element>,
    document: Trusted<Document>,
    promise: TrustedPromise,
    error: bool,
}

impl ElementPerformFullscreenEnter {
    fn new(
        element: Trusted<Element>,
        document: Trusted<Document>,
        promise: TrustedPromise,
        error: bool,
    ) -> Box<ElementPerformFullscreenEnter> {
        Box::new(ElementPerformFullscreenEnter {
            element,
            document,
            promise,
            error,
        })
    }
}

impl TaskOnce for ElementPerformFullscreenEnter {
    /// Step 9-14 of <https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen>
    fn run_once(self, cx: &mut js::context::JSContext) {
        let element = self.element.root();
        let promise = self.promise.root(cx);
        let document = element.owner_document();

        // Step 9
        // > If any of the following conditions are false, then set error to true:
        // > - This’s node document is pendingDoc.
        // > - The fullscreen element ready check for this returns true.
        // Step 10
        // > If error is true:
        // > - Append (fullscreenerror, this) to pendingDoc’s list of pending fullscreen events.
        // > - Reject promise with a TypeError exception and terminate these steps.
        if self.document.root() != document ||
            !element.fullscreen_element_ready_check() ||
            self.error
        {
            // TODO(#31866): we should queue this and fire them in update the rendering.
            document
                .upcast::<EventTarget>()
                .fire_event(cx, atom!("fullscreenerror"));
            promise.reject_error(cx, Error::Type(c"fullscreen is not connected".to_owned()));
            return;
        }

        // TODO(#42067): Implement step 11-13
        // The following operations is based on the old version of the specs.
        element.set_fullscreen_state(cx.no_gc(), true);
        document
            .overlay_handler()
            .set_fullscreen_element(Some(&element));
        document.upcast::<EventTarget>().fire_event_with_params(
            cx,
            atom!("fullscreenchange"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
        );

        // Step 14.
        // > Resolve promise with undefined.
        promise.resolve_native(cx, &());
    }
}

struct ElementPerformFullscreenExit {
    element: Trusted<Element>,
    promise: TrustedPromise,
}

impl ElementPerformFullscreenExit {
    fn new(
        element: Trusted<Element>,
        promise: TrustedPromise,
    ) -> Box<ElementPerformFullscreenExit> {
        Box::new(ElementPerformFullscreenExit { element, promise })
    }
}

impl TaskOnce for ElementPerformFullscreenExit {
    /// Step 9-16 of <https://fullscreen.spec.whatwg.org/#exit-fullscreen>
    fn run_once(self, cx: &mut js::context::JSContext) {
        let element = self.element.root();
        let document = element.owner_document();
        // Step 9.
        // > Run the fully unlock the screen orientation steps with doc.
        // TODO: Need to implement ScreenOrientation API first

        // TODO(#42067): Implement step 10-15
        // The following operations is based on the old version of the specs.
        element.set_fullscreen_state(cx.no_gc(), false);
        document.overlay_handler().set_fullscreen_element(None);
        document.upcast::<EventTarget>().fire_event_with_params(
            cx,
            atom!("fullscreenchange"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
        );

        // Step 16
        // > Resolve promise with undefined.
        self.promise.root(cx).resolve_native(cx, &());
    }
}
