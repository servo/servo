/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding;
use dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding::DissimilarOriginWindowMethods;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::browsingcontext::BrowsingContext;
use dom::dissimilaroriginlocation::DissimilarOriginLocation;
use dom::globalscope::GlobalScope;
use ipc_channel::ipc;
use js::jsapi::{JSContext, HandleValue};
use js::jsval::{JSVal, UndefinedValue};
use msg::constellation_msg::PipelineId;

/// Represents a dissimilar-origin `Window` that exists in another script thread.
///
/// Since the `Window` is in a different script thread, we cannot access it
/// directly, but some of its accessors (for example `window.parent`)
/// still need to function.
///
/// In `browsingcontext.rs`, we create a custom window proxy for these windows,
/// that throws security exceptions for most accessors. This is not a replacement
/// for XOWs, but provides belt-and-braces security.
#[dom_struct]
pub struct DissimilarOriginWindow {
    /// The global for this window.
    globalscope: GlobalScope,

    /// The browsing context this window is part of.
    browsing_context: JS<BrowsingContext>,

    /// The location of this window, initialized lazily.
    location: MutNullableJS<DissimilarOriginLocation>,
}

impl DissimilarOriginWindow {
    #[allow(unsafe_code)]
    pub fn new(browsing_context: &BrowsingContext) -> Root<DissimilarOriginWindow> {
        let globalscope = browsing_context.global();
        let cx = globalscope.get_cx();
        // Any timer events fired on this window are ignored.
        let (timer_event_chan, _) = ipc::channel().unwrap();
        let win = box DissimilarOriginWindow {
            globalscope: GlobalScope::new_inherited(PipelineId::new(),
                                                    globalscope.devtools_chan().cloned(),
                                                    globalscope.mem_profiler_chan().clone(),
                                                    globalscope.time_profiler_chan().clone(),
                                                    globalscope.constellation_chan().clone(),
                                                    globalscope.scheduler_chan().clone(),
                                                    globalscope.resource_threads().clone(),
                                                    timer_event_chan),
            browsing_context: JS::from_ref(browsing_context),
            location: MutNullableJS::new(None),
        };
        unsafe { DissimilarOriginWindowBinding::Wrap(cx, win) }
    }
}

impl DissimilarOriginWindowMethods for DissimilarOriginWindow {
    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> Root<BrowsingContext> {
        Root::from_ref(&*self.browsing_context)
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> Root<BrowsingContext> {
        Root::from_ref(&*self.browsing_context)
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> Root<BrowsingContext> {
        Root::from_ref(&*self.browsing_context)
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn GetParent(&self) -> Option<Root<BrowsingContext>> {
        // TODO: implement window.parent correctly for x-origin windows.
        Some(Root::from_ref(&*self.browsing_context))
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn GetTop(&self) -> Option<Root<BrowsingContext>> {
        // TODO: implement window.top correctly for x-origin windows.
        Some(Root::from_ref(&*self.browsing_context))
    }

    // https://html.spec.whatwg.org/multipage/#dom-length
    fn Length(&self) -> u32 {
        // TODO: Implement x-origin length
        0
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-close
    fn Close(&self) {
        // TODO: Implement x-origin close
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-closed
    fn Closed(&self) -> bool {
        // TODO: Implement x-origin close
        false
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage
    unsafe fn PostMessage(&self, _: *mut JSContext, _: HandleValue, _: DOMString) {
        // TODO: Implement x-origin postMessage
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-opener
    unsafe fn Opener(&self, _: *mut JSContext) -> JSVal {
        // TODO: Implement x-origin opener
        UndefinedValue()
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-opener
    unsafe fn SetOpener(&self, _: *mut JSContext, _: HandleValue) {
        // TODO: Implement x-origin opener
    }

    // https://html.spec.whatwg.org/multipage/#dom-window-blur
    fn Blur(&self) {
        // TODO: Implement x-origin blur
    }

    // https://html.spec.whatwg.org/multipage/#dom-focus
    fn Focus(&self) {
        // TODO: Implement x-origin focus
    }

    // https://html.spec.whatwg.org/multipage/#dom-location
    fn Location(&self) -> Root<DissimilarOriginLocation> {
        self.location.or_init(|| DissimilarOriginLocation::new(self))
    }
}
