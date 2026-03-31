/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use embedder_traits::EmbedderMsg;
use html5ever::{local_name, ns};
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::document::Document;
use crate::dom::document::documentorshadowroot::DocumentOrShadowRoot;
use crate::dom::element::Element;
use crate::dom::event::event::{EventBubbles, EventCancelable, EventComposed};
use crate::dom::event::eventtarget::EventTarget;
use crate::dom::node::NodeTraits;
use crate::dom::node::node::Node;
use crate::dom::promise::Promise;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::types::HTMLDialogElement;
use crate::messaging::{CommonScriptMsg, MainThreadScriptMsg};
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, ScriptThreadEventCategory};
use crate::task::TaskOnce;
use crate::task_source::TaskSourceName;

impl Document {
    /// <https://fullscreen.spec.whatwg.org/#dom-element-requestfullscreen>
    pub(crate) fn enter_fullscreen(&self, pending: &Element, can_gc: CanGc) -> Rc<Promise> {
        // Step 1
        // > Let pendingDoc be this’s node document.
        // `Self` is the pending document.

        // Step 2
        // > Let promise be a new promise.
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 3
        // > If pendingDoc is not fully active, then reject promise with a TypeError exception and return promise.
        if !self.is_fully_active() {
            promise.reject_error(
                Error::Type(c"Document is not fully active".to_owned()),
                can_gc,
            );
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
                    if pending.local_name().as_ref() != "math" {
                        error = true;
                    }
                },
                ns!(svg) => {
                    if pending.local_name().as_ref() != "svg" {
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
            pending.owner_window().consume_user_activation();
        }

        // Step 8.
        // > If error is false, then resize pendingDoc’s node navigable’s top-level traversable’s active document’s viewport’s dimensions,
        // > optionally taking into account options["navigationUI"]:
        // TODO(#21600): Improve spec compliance of steps 7-13 paralelism.
        // TODO(#42064): Implement fullscreen options, and ensure that this is spec compliant for all embedder.
        if !error {
            let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), true);
            self.send_to_embedder(event);
        }

        // Step 7
        // > Return promise, and run the remaining steps in parallel.
        let pipeline_id = self.window().pipeline_id();

        let trusted_pending = Trusted::new(pending);
        let trusted_pending_doc = Trusted::new(self);
        let trusted_promise = TrustedPromise::new(promise.clone());
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
        self.window().main_thread_script_chan().send(msg).unwrap();

        promise
    }

    /// <https://fullscreen.spec.whatwg.org/#exit-fullscreen>
    pub(crate) fn exit_fullscreen(&self, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();

        // Step 1
        // > Let promise be a new promise
        let in_realm_proof = AlreadyInRealm::assert::<crate::DomTypeHolder>();
        let promise = Promise::new_in_current_realm(InRealm::Already(&in_realm_proof), can_gc);

        // Step 2
        // > If doc is not fully active or doc’s fullscreen element is null, then reject promise with a TypeError exception and return promise.
        if !self.is_fully_active() || self.fullscreen_element().is_none() {
            promise.reject_error(
                Error::Type(
                    c"No fullscreen element to exit or document is not fully active".to_owned(),
                ),
                can_gc,
            );
            return promise;
        }

        // TODO(#42067): Implement step 3-7, handling fullscreen's propagation across navigables.

        let element = self.fullscreen_element().unwrap();
        let window = self.window();

        // Step 10
        // > If resize is true, resize doc’s viewport to its "normal" dimensions.
        // TODO(#21600): Improve spec compliance of steps 8-15 paralelism.
        let event = EmbedderMsg::NotifyFullscreenStateChanged(self.webview_id(), false);
        self.send_to_embedder(event);

        // Step 8
        // > Return promise, and run the remaining steps in parallel.
        let trusted_element = Trusted::new(&*element);
        let trusted_promise = TrustedPromise::new(promise.clone());
        let handler = ElementPerformFullscreenExit::new(trusted_element, trusted_promise);
        let pipeline_id = Some(global.pipeline_id());
        let script_msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::ExitFullscreen,
            handler,
            pipeline_id,
            TaskSourceName::DOMManipulation,
        );
        let msg = MainThreadScriptMsg::Common(script_msg);
        window.main_thread_script_chan().send(msg).unwrap();

        promise
    }

    pub(crate) fn get_allow_fullscreen(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#allowed-to-use
        match self.browsing_context() {
            // Step 1
            None => false,
            Some(_) => {
                // Step 2
                let window = self.window();
                if window.is_top_level() {
                    true
                } else {
                    // Step 3
                    window
                        .GetFrameElement()
                        .is_some_and(|el| el.has_attribute(&local_name!("allowfullscreen")))
                }
            },
        }
    }
}

impl DocumentOrShadowRoot {
    /// <https://fullscreen.spec.whatwg.org/#dom-document-fullscreenelement>
    pub(crate) fn get_fullscreen_element(
        node: &Node,
        fullscreen_element: Option<DomRoot<Element>>,
    ) -> Option<DomRoot<Element>> {
        // Step 1. If this is a shadow root and its host is not connected, then return null.
        if let Some(shadow_root) = node.downcast::<ShadowRoot>() {
            if !shadow_root.Host().is_connected() {
                return None;
            }
        }

        // Step 2. Let candidate be the result of retargeting fullscreen element against this.
        let retargeted = fullscreen_element?
            .upcast::<EventTarget>()
            .retarget(node.upcast());
        // It's safe to unwrap downcasting to `Element` because `retarget` either returns `fullscreen_element` or a host of `fullscreen_element` and hosts are always elements.
        let candidate = DomRoot::downcast::<Element>(retargeted).unwrap();

        // Step 3. If candidate and this are in the same tree, then return candidate.
        if *candidate
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty()) ==
            *node
        {
            return Some(candidate);
        }

        // Step 4. Return null.
        None
    }
}

impl Element {
    // https://fullscreen.spec.whatwg.org/#fullscreen-element-ready-check
    pub(crate) fn fullscreen_element_ready_check(&self) -> bool {
        if !self.is_connected() {
            return false;
        }
        self.owner_document().get_allow_fullscreen()
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
        let promise = self.promise.root();
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
                .fire_event(atom!("fullscreenerror"), CanGc::from_cx(cx));
            promise.reject_error(
                Error::Type(c"fullscreen is not connected".to_owned()),
                CanGc::from_cx(cx),
            );
            return;
        }

        // TODO(#42067): Implement step 11-13
        // The following operations is based on the old version of the specs.
        element.set_fullscreen_state(true);
        document.set_fullscreen_element(Some(&element));
        document.upcast::<EventTarget>().fire_event_with_params(
            atom!("fullscreenchange"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            CanGc::from_cx(cx),
        );

        // Step 14.
        // > Resolve promise with undefined.
        promise.resolve_native(&(), CanGc::from_cx(cx));
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
        element.set_fullscreen_state(false);
        document.set_fullscreen_element(None);
        document.upcast::<EventTarget>().fire_event_with_params(
            atom!("fullscreenchange"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            EventComposed::Composed,
            CanGc::from_cx(cx),
        );

        // Step 16
        // > Resolve promise with undefined.
        self.promise.root().resolve_native(&(), CanGc::from_cx(cx));
    }
}
