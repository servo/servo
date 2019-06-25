/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding;
use crate::dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding::DissimilarOriginWindowMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowPostMessageOptions;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::dissimilaroriginlocation::DissimilarOriginLocation;
use crate::dom::globalscope::GlobalScope;
use crate::dom::windowproxy::WindowProxy;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use msg::constellation_msg::PipelineId;
use script_traits::{ScriptMsg, StructuredSerializedData};
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
    window_proxy: Dom<WindowProxy>,

    /// The location of this window, initialized lazily.
    location: MutNullableDom<DissimilarOriginLocation>,
}

impl DissimilarOriginWindow {
    #[allow(unsafe_code)]
    pub fn new(global_to_clone_from: &GlobalScope, window_proxy: &WindowProxy) -> DomRoot<Self> {
        let cx = global_to_clone_from.get_cx();
        // Any timer events fired on this window are ignored.
        let (timer_event_chan, _) = ipc::channel().unwrap();
        let win = Box::new(Self {
            globalscope: GlobalScope::new_inherited(
                PipelineId::new(),
                global_to_clone_from.devtools_chan().cloned(),
                global_to_clone_from.mem_profiler_chan().clone(),
                global_to_clone_from.time_profiler_chan().clone(),
                global_to_clone_from.script_to_constellation_chan().clone(),
                global_to_clone_from.scheduler_chan().clone(),
                global_to_clone_from.resource_threads().clone(),
                timer_event_chan,
                global_to_clone_from.origin().clone(),
                // FIXME(nox): The microtask queue is probably not important
                // here, but this whole DOM interface is a hack anyway.
                global_to_clone_from.microtask_queue().clone(),
                global_to_clone_from.is_headless(),
                global_to_clone_from.get_user_agent(),
            ),
            window_proxy: Dom::from_ref(window_proxy),
            location: Default::default(),
        });
        unsafe { DissimilarOriginWindowBinding::Wrap(cx, win) }
    }

    pub fn window_proxy(&self) -> DomRoot<WindowProxy> {
        DomRoot::from_ref(&*self.window_proxy)
    }
}

impl DissimilarOriginWindowMethods for DissimilarOriginWindow {
    // https://html.spec.whatwg.org/multipage/#dom-window
    fn Window(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-self
    fn Self_(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-frames
    fn Frames(&self) -> DomRoot<WindowProxy> {
        self.window_proxy()
    }

    // https://html.spec.whatwg.org/multipage/#dom-parent
    fn GetParent(&self) -> Option<DomRoot<WindowProxy>> {
        // Steps 1-3.
        if self.window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Step 4.
        if let Some(parent) = self.window_proxy.parent() {
            return Some(DomRoot::from_ref(parent));
        }
        // Step 5.
        Some(DomRoot::from_ref(&*self.window_proxy))
    }

    // https://html.spec.whatwg.org/multipage/#dom-top
    fn GetTop(&self) -> Option<DomRoot<WindowProxy>> {
        // Steps 1-3.
        if self.window_proxy.is_browsing_context_discarded() {
            return None;
        }
        // Steps 4-5.
        Some(DomRoot::from_ref(self.window_proxy.top()))
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

    // https://html.spec.whatwg.org/multipage/#dom-window-postmessage
    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        origin: USVString,
        mut transfer: CustomAutoRooterGuard<Option<Vec<*mut JSObject>>>,
    ) -> ErrorResult {
        if transfer.is_some() {
            let mut rooted = CustomAutoRooter::new(transfer.take().unwrap());
            let transfer = Some(CustomAutoRooterGuard::new(*cx, &mut rooted));
            self.post_message_impl(&Some(origin), cx, message, transfer)
        } else {
            self.post_message_impl(&Some(origin), cx, message, None)
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage_(
        &self,
        cx: JSContext,
        message: HandleValue,
        options: RootedTraceableBox<WindowPostMessageOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .as_ref()
                .unwrap_or(&Vec::with_capacity(0))
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let transfer = Some(CustomAutoRooterGuard::new(*cx, &mut rooted));

        self.post_message_impl(&options.targetOrigin, cx, message, transfer)
    }

    // https://html.spec.whatwg.org/multipage/#dom-opener
    fn Opener(&self, _: JSContext) -> JSVal {
        // TODO: Implement x-origin opener
        UndefinedValue()
    }

    // https://html.spec.whatwg.org/multipage/#dom-opener
    fn SetOpener(&self, _: JSContext, _: HandleValue) {
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
    fn Location(&self) -> DomRoot<DissimilarOriginLocation> {
        self.location
            .or_init(|| DissimilarOriginLocation::new(self))
    }
}

impl DissimilarOriginWindow {
    fn post_message_impl(
        &self,
        target_origin: &Option<USVString>,
        cx: JSContext,
        message: HandleValue,
        transfer: Option<CustomAutoRooterGuard<Vec<*mut JSObject>>>,
    ) -> ErrorResult {
        // Step 1-2, 6-8.
        let data = structuredclone::write(*cx, message, transfer)?;

        // Step 9.
        self.post_message(target_origin, data);
        Ok(())
    }

    pub fn post_message(&self, target_origin: &Option<USVString>, data: StructuredSerializedData) {
        let incumbent = match GlobalScope::incumbent() {
            None => return warn!("postMessage called with no incumbent global"),
            Some(incumbent) => incumbent,
        };

        let source_origin = incumbent.origin().immutable().clone();

        // Step 3-5.
        let target_origin = match &target_origin {
            Some(origin) => match origin.0[..].as_ref() {
                "*" => None,
                "/" => Some(source_origin.clone()),
                url => match ServoUrl::parse(&url) {
                    Ok(url) => Some(url.origin().clone()),
                    Err(_) => return warn!("Syntax error in target-origin string"),
                },
            },
            None => Some(source_origin.clone()),
        };
        let msg = ScriptMsg::PostMessage {
            target: self.window_proxy.browsing_context_id(),
            source: incumbent.pipeline_id(),
            source_origin,
            target_origin,
            data: data,
        };
        let _ = incumbent.script_to_constellation_chan().send(msg);
    }
}
