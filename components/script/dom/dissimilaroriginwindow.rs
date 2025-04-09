/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PipelineId;
use constellation_traits::{ScriptToConstellationMessage, StructuredSerializedData};
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::jsval::UndefinedValue;
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue, MutableHandleValue};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding;
use crate::dom::bindings::codegen::Bindings::DissimilarOriginWindowBinding::DissimilarOriginWindowMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowPostMessageOptions;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::dissimilaroriginlocation::DissimilarOriginLocation;
use crate::dom::globalscope::GlobalScope;
use crate::dom::windowproxy::WindowProxy;
use crate::script_runtime::{CanGc, JSContext};

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
pub(crate) struct DissimilarOriginWindow {
    /// The global for this window.
    globalscope: GlobalScope,

    /// The window proxy for this window.
    window_proxy: Dom<WindowProxy>,

    /// The location of this window, initialized lazily.
    location: MutNullableDom<DissimilarOriginLocation>,
}

impl DissimilarOriginWindow {
    #[allow(unsafe_code)]
    pub(crate) fn new(
        global_to_clone_from: &GlobalScope,
        window_proxy: &WindowProxy,
    ) -> DomRoot<Self> {
        let cx = GlobalScope::get_cx();
        let win = Box::new(Self {
            globalscope: GlobalScope::new_inherited(
                PipelineId::new(),
                global_to_clone_from.devtools_chan().cloned(),
                global_to_clone_from.mem_profiler_chan().clone(),
                global_to_clone_from.time_profiler_chan().clone(),
                global_to_clone_from.script_to_constellation_chan().clone(),
                global_to_clone_from.resource_threads().clone(),
                global_to_clone_from.origin().clone(),
                global_to_clone_from.creation_url().clone(),
                // FIXME(nox): The microtask queue is probably not important
                // here, but this whole DOM interface is a hack anyway.
                global_to_clone_from.microtask_queue().clone(),
                #[cfg(feature = "webgpu")]
                global_to_clone_from.wgpu_id_hub(),
                Some(global_to_clone_from.is_secure_context()),
                false,
            ),
            window_proxy: Dom::from_ref(window_proxy),
            location: Default::default(),
        });
        unsafe { DissimilarOriginWindowBinding::Wrap::<crate::DomTypeHolder>(cx, win) }
    }

    pub(crate) fn window_proxy(&self) -> DomRoot<WindowProxy> {
        DomRoot::from_ref(&*self.window_proxy)
    }
}

impl DissimilarOriginWindowMethods<crate::DomTypeHolder> for DissimilarOriginWindow {
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

    /// <https://html.spec.whatwg.org/multipage/#dom-window-postmessage>
    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        target_origin: USVString,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        self.post_message_impl(&target_origin, cx, message, transfer)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-window-postmessage-options>
    fn PostMessage_(
        &self,
        cx: JSContext,
        message: HandleValue,
        options: RootedTraceableBox<WindowPostMessageOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .parent
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let transfer = CustomAutoRooterGuard::new(*cx, &mut rooted);

        self.post_message_impl(&options.targetOrigin, cx, message, transfer)
    }

    // https://html.spec.whatwg.org/multipage/#dom-opener
    fn Opener(&self, _: JSContext, mut retval: MutableHandleValue) {
        // TODO: Implement x-origin opener
        retval.set(UndefinedValue());
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
    fn Location(&self, can_gc: CanGc) -> DomRoot<DissimilarOriginLocation> {
        self.location
            .or_init(|| DissimilarOriginLocation::new(self, can_gc))
    }
}

impl DissimilarOriginWindow {
    /// <https://html.spec.whatwg.org/multipage/#window-post-message-steps>
    fn post_message_impl(
        &self,
        target_origin: &USVString,
        cx: JSContext,
        message: HandleValue,
        transfer: CustomAutoRooterGuard<Vec<*mut JSObject>>,
    ) -> ErrorResult {
        // Step 6-7.
        let data = structuredclone::write(cx, message, Some(transfer))?;

        self.post_message(target_origin, data)
    }

    /// <https://html.spec.whatwg.org/multipage/#window-post-message-steps>
    pub(crate) fn post_message(
        &self,
        target_origin: &USVString,
        data: StructuredSerializedData,
    ) -> ErrorResult {
        // Step 1.
        let target = self.window_proxy.browsing_context_id();
        // Step 2.
        let incumbent = match GlobalScope::incumbent() {
            None => panic!("postMessage called with no incumbent global"),
            Some(incumbent) => incumbent,
        };

        let source_origin = incumbent.origin().immutable().clone();

        // Step 3-5.
        let target_origin = match target_origin.0[..].as_ref() {
            "*" => None,
            "/" => Some(source_origin.clone()),
            url => match ServoUrl::parse(url) {
                Ok(url) => Some(url.origin().clone()),
                Err(_) => return Err(Error::Syntax),
            },
        };
        let msg = ScriptToConstellationMessage::PostMessage {
            target,
            source: incumbent.pipeline_id(),
            source_origin,
            target_origin,
            data,
        };
        // Step 8
        let _ = incumbent.script_to_constellation_chan().send(msg);
        Ok(())
    }
}
