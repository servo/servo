/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding;
use dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding::DissimilarOriginWindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::dissimilaroriginlocation::DissimilarOriginLocation;
use dom::globalscope::GlobalScope;
use dom::windowproxy::WindowProxy;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{JSContext, HandleValue};
use js::jsval::{JSVal, UndefinedValue};
use msg::constellation_msg::PipelineId;
use script_traits::ScriptMsg as ConstellationMsg;
use servo_url::ImmutableOrigin;
use servo_url::MutableOrigin;
use servo_url::ServoUrl;

/// Represents a dissimilar-origin `Window` that exists in another script thread.
///
/// Since the `Window` is in a different script thread, we cannot access it
/// directly, but some of its accessors (for example `window.parent`)
/// still need to function.
///
/// In `windowproxy.rs`, we create a custom window proxy for these windows,
/// that throws security exceptions for most accessors. This is not a replacement
/// for XOWs, but provides belt-and-braces security.
#[dom_struct]
pub struct DissimilarOriginWindow {
    /// The global for this window.
    globalscope: GlobalScope,

    /// The window proxy for this window.
    window_proxy: JS<WindowProxy>,

    /// The location of this window, initialized lazily.
    location: MutNullableJS<DissimilarOriginLocation>,
}

impl DissimilarOriginWindow {
    #[allow(unsafe_code)]
    pub fn new(global_to_clone_from: &GlobalScope, window_proxy: &WindowProxy) -> Root<DissimilarOriginWindow> {
        let cx = global_to_clone_from.get_cx();
        // Any timer events fired on this window are ignored.
        let (timer_event_chan, _) = ipc::channel().unwrap();
        let win = box DissimilarOriginWindow {
            globalscope: GlobalScope::new_inherited(PipelineId::new(),
                                                    global_to_clone_from.devtools_chan().cloned(),
                                                    global_to_clone_from.mem_profiler_chan().clone(),
                                                    global_to_clone_from.time_profiler_chan().clone(),
                                                    global_to_clone_from.constellation_chan().clone(),
                                                    global_to_clone_from.scheduler_chan().clone(),
                                                    global_to_clone_from.resource_threads().clone(),
                                                    timer_event_chan,
                                                    global_to_clone_from.origin().clone()),
            window_proxy: JS::from_ref(window_proxy),
            location: MutNullableJS::new(None),
        };
        unsafe { DissimilarOriginWindowBinding::Wrap(cx, win) }
    }

    #[allow(dead_code)]
    pub fn origin(&self) -> &MutableOrigin {
        self.globalscope.origin()
    }
}

impl DissimilarOriginWindowMethods for DissimilarOriginWindow {
    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> Root<WindowProxy> {
        Root::from_ref(&*self.window_proxy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> Root<WindowProxy> {
        Root::from_ref(&*self.window_proxy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> Root<WindowProxy> {
        Root::from_ref(&*self.window_proxy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn GetParent(&self) -> Option<Root<WindowProxy>> {
        // Steps 1-3.
        if self.window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Step 4.
        if let Some(parent) = self.window_proxy.parent() {
            return Some(Root::from_ref(parent));
        }
        // Step 5.
        Some(Root::from_ref(&*self.window_proxy))
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn GetTop(&self) -> Option<Root<WindowProxy>> {
        // Steps 1-3.
        if self.window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Steps 4-5.
        Some(Root::from_ref(self.window_proxy.top()))
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
    unsafe fn PostMessage(&self, cx: *mut JSContext, message: HandleValue, origin: DOMString) -> ErrorResult {
        // Step 3-5.
        let origin = match &origin[..] {
            "*" => None,
            "/" => {
                // TODO: Should be the origin of the incumbent settings object.
                None
            },
            url => match ServoUrl::parse(&url) {
                Ok(url) => Some(url.origin()),
                Err(_) => return Err(Error::Syntax),
            }
        };

        // Step 1-2, 6-8.
        // TODO(#12717): Should implement the `transfer` argument.
        let data = StructuredCloneData::write(cx, message)?;

        // Step 9.
        self.post_message(origin, data);
        Ok(())
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

impl DissimilarOriginWindow {
    pub fn post_message(&self, origin: Option<ImmutableOrigin>, data: StructuredCloneData) {
        let msg = ConstellationMsg::PostMessage(self.window_proxy.browsing_context_id(),
                                                origin,
                                                data.move_to_arraybuffer());
        let _ = self.upcast::<GlobalScope>().constellation_chan().send(msg);
    }
}
