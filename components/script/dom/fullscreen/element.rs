/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::document::document::Document;
use crate::dom::element::element::Element;
use crate::dom::event::event::{EventBubbles, EventCancelable, EventComposed};
use crate::dom::event::eventtarget::EventTarget;
use crate::dom::node::NodeTraits;
use crate::script_runtime::CanGc;
use crate::task::TaskOnce;

impl Element {
    // https://fullscreen.spec.whatwg.org/#fullscreen-element-ready-check
    pub(crate) fn fullscreen_element_ready_check(&self) -> bool {
        if !self.is_connected() {
            return false;
        }
        self.owner_document().get_allow_fullscreen()
    }
}

pub(crate) struct ElementPerformFullscreenEnter {
    element: Trusted<Element>,
    document: Trusted<Document>,
    promise: TrustedPromise,
    error: bool,
}

impl ElementPerformFullscreenEnter {
    pub(crate) fn new(
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

pub(crate) struct ElementPerformFullscreenExit {
    element: Trusted<Element>,
    promise: TrustedPromise,
}

impl ElementPerformFullscreenExit {
    pub(crate) fn new(
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
