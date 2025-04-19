/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::HandleValue as SafeHandleValue;

use super::bindings::root::DomRoot;
use super::types::TransformStream;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::Dom;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};

#[dom_struct]
/// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
pub(crate) struct TransformUnderlyingSource {
    reflector_: Reflector,
    stream: Dom<TransformStream>,
    #[ignore_malloc_size_of = "Rc is hard"]
    start_promise: Rc<Promise>,
}

impl TransformUnderlyingSource {
    #[allow(clippy::redundant_allocation)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        stream: &TransformStream,
        start_promise: Rc<Promise>,
        can_gc: CanGc,
    ) -> DomRoot<TransformUnderlyingSource> {
        reflect_dom_object(
            Box::new(TransformUnderlyingSource {
                reflector_: Reflector::new(),
                stream: Dom::from_ref(stream),
                start_promise,
            }),
            &*stream.global(),
            can_gc,
        )
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    pub(crate) fn start_algorithm(&self) -> Fallible<Rc<Promise>> {
        // Let startAlgorithm be an algorithm that returns startPromise.
        Ok(self.start_promise.clone())
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    /// Let writeAlgorithm be the following steps, taking a chunk argument:
    pub(crate) fn write_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Return ! TransformStreamDefaultSinkWriteAlgorithm(stream, chunk).
        self.stream
            .default_sink_write_algorithm(cx, global, chunk, can_gc)
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    /// Let abortAlgorithm be the following steps, taking a reason argument:
    pub(crate) fn abort_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Return ! TransformStreamDefaultSinkAbortAlgorithm(stream, reason).
        self.stream
            .default_sink_abort_algorithm(cx, global, reason, can_gc)
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    /// Let closeAlgorithm be the following steps:
    pub(crate) fn close_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Return ! TransformStreamDefaultSinkCloseAlgorithm(stream).
        self.stream.default_sink_close_algorithm(cx, global, can_gc)
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    /// Let cancelAlgorithm be the following steps, taking a reason argument:
    pub(crate) fn cancel_algorithm(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Return ! TransformStreamDefaultSourceCancelAlgorithm(stream, reason).
        self.stream
            .default_source_cancel_algorithm(cx, global, reason, can_gc)
    }

    /// continuation of <https://streams.spec.whatwg.org/#initialize-transform-stream>
    /// Let pullAlgorithm be the following steps:
    pub(crate) fn pull_algorithm(
        &self,
        global: &GlobalScope,
        can_gc: CanGc,
    ) -> Fallible<Rc<Promise>> {
        // Return ! TransformStreamDefaultSourcePullAlgorithm(stream).
        self.stream.default_source_pull_algorithm(global, can_gc)
    }
}
