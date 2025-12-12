/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ptr::{self};
use std::rc::Rc;

use base::id::{MessagePortId, MessagePortIndex};
use constellation_traits::MessagePortImpl;
use dom_struct::dom_struct;
use base::generic_channel::GenericSharedMemory;
use js::jsapi::{Heap, JSObject};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::{
    HandleObject as SafeHandleObject, HandleValue as SafeHandleValue,
    MutableHandleValue as SafeMutableHandleValue,
};
use js::typedarray::ArrayBufferViewU8;
use rustc_hash::FxHashMap;
use script_bindings::conversions::SafeToJSValConvertible;

use crate::dom::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategy;
use crate::dom::bindings::codegen::Bindings::ReadableStreamBinding::{
    ReadableStreamGetReaderOptions, ReadableStreamMethods, ReadableStreamReaderMode,
    ReadableWritablePair, StreamPipeOptions,
};
use script_bindings::str::DOMString;

use crate::dom::domexception::{DOMErrorName, DOMException};
use script_bindings::conversions::{is_array_like, StringificationBehavior};
use super::bindings::codegen::Bindings::QueuingStrategyBinding::QueuingStrategySize;
use crate::dom::abortsignal::{AbortAlgorithm, AbortSignal};
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultReaderBinding::ReadableStreamDefaultReaderMethods;
use crate::dom::bindings::codegen::Bindings::ReadableStreamDefaultControllerBinding::ReadableStreamDefaultController_Binding::ReadableStreamDefaultControllerMethods;
use crate::dom::bindings::codegen::Bindings::UnderlyingSourceBinding::UnderlyingSource as JsUnderlyingSource;
use crate::dom::bindings::conversions::{ConversionBehavior, ConversionResult, SafeFromJSValConvertible};
use crate::dom::bindings::error::{Error, ErrorToJsval, Fallible};
use crate::dom::bindings::codegen::GenericBindings::WritableStreamDefaultWriterBinding::WritableStreamDefaultWriter_Binding::WritableStreamDefaultWriterMethods;
use crate::dom::writablestream::WritableStream;
use crate::dom::bindings::codegen::UnionTypes::ReadableStreamDefaultReaderOrReadableStreamBYOBReader as ReadableStreamReader;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{DomRoot, MutNullableDom, Dom};
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::get_dictionary_property;
use crate::dom::byteteeunderlyingsource::{ByteTeeCancelAlgorithm, ByteTeePullAlgorithm, ByteTeeUnderlyingSource};
use crate::dom::countqueuingstrategy::{extract_high_water_mark, extract_size_algorithm};
use crate::dom::readablestreamgenericreader::ReadableStreamGenericReader;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::{wait_for_all_promise, Promise};
use crate::dom::readablebytestreamcontroller::ReadableByteStreamController;
use crate::dom::readablestreambyobreader::ReadableStreamBYOBReader;
use crate::dom::readablestreamdefaultcontroller::ReadableStreamDefaultController;
use crate::dom::readablestreamdefaultreader::{ReadRequest, ReadableStreamDefaultReader};
use crate::dom::defaultteeunderlyingsource::DefaultTeeCancelAlgorithm;
use crate::dom::types::DefaultTeeUnderlyingSource;
use crate::dom::underlyingsourcecontainer::UnderlyingSourceType;
use crate::dom::writablestreamdefaultwriter::WritableStreamDefaultWriter;
use script_bindings::codegen::GenericBindings::MessagePortBinding::MessagePortMethods;
use crate::dom::messageport::MessagePort;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::bindings::transferable::Transferable;
use crate::dom::bindings::structuredclone::StructuredData;

use super::bindings::buffer_source::HeapBufferSource;
use super::readablestreambyobreader::ReadIntoRequest;

/// State Machine for `PipeTo`.
#[derive(Clone, Debug, Default, MallocSizeOf, PartialEq)]
enum PipeToState {
    /// The starting state
    #[default]
    Starting,
    /// Waiting for the writer to be ready
    PendingReady,
    /// Waiting for a read to resolve.
    PendingRead,
    /// Waiting for all pending writes to finish,
    /// as part of shutting down with an optional action.
    ShuttingDownWithPendingWrites(Option<ShutdownAction>),
    /// When shutting down with an action,
    /// waiting for the action to complete,
    /// at which point we can `finalize`.
    ShuttingDownPendingAction,
    /// The pipe has been finalized,
    /// no further actions should be performed.
    Finalized,
}

/// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown-with-action>
#[derive(Clone, Debug, MallocSizeOf, PartialEq)]
enum ShutdownAction {
    /// <https://streams.spec.whatwg.org/#writable-stream-abort>
    WritableStreamAbort,
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    ReadableStreamCancel,
    /// <https://streams.spec.whatwg.org/#writable-stream-default-writer-close-with-error-propagation>
    WritableStreamDefaultWriterCloseWithErrorPropagation,
    /// <https://streams.spec.whatwg.org/#ref-for-rs-pipeTo-shutdown-with-action>
    Abort,
}

impl js::gc::Rootable for PipeTo {}

/// The "in parallel, but not really" part of
/// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
///
/// Note: the spec is flexible about how this is done, but requires the following constraints to apply:
/// - Public API must not be used: we'll only use Rust.
/// - Backpressure must be enforced: we'll only read from source when dest is ready.
/// - Shutdown must stop activity: we'll do this together with the below.
/// - Error and close states must be propagated: we'll do this by checking these states at every step.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct PipeTo {
    /// <https://streams.spec.whatwg.org/#ref-for-readablestream%E2%91%A7%E2%91%A0>
    reader: Dom<ReadableStreamDefaultReader>,

    /// <https://streams.spec.whatwg.org/#ref-for-acquire-writable-stream-default-writer>
    writer: Dom<WritableStreamDefaultWriter>,

    /// Pending writes are needed when shutting down(with an action),
    /// because we can only finalize when all writes are finished.
    #[ignore_malloc_size_of = "nested Rc"]
    pending_writes: Rc<RefCell<VecDeque<Rc<Promise>>>>,

    /// The state machine.
    #[conditional_malloc_size_of]
    #[no_trace]
    state: Rc<RefCell<PipeToState>>,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventabort>
    prevent_abort: bool,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventcancel>
    prevent_cancel: bool,

    /// <https://streams.spec.whatwg.org/#readablestream-pipe-to-preventclose>
    prevent_close: bool,

    /// The `shuttingDown` variable of
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    #[conditional_malloc_size_of]
    shutting_down: Rc<Cell<bool>>,

    /// The abort reason of the abort signal,
    /// stored here because we must keep it across a microtask.
    #[ignore_malloc_size_of = "mozjs"]
    abort_reason: Rc<Heap<JSVal>>,

    /// The error potentially passed to shutdown,
    /// stored here because we must keep it across a microtask.
    #[ignore_malloc_size_of = "mozjs"]
    shutdown_error: Rc<RefCell<Option<Heap<JSVal>>>>,

    /// The promise returned by a shutdown action.
    /// We keep it to only continue when it is not pending anymore.
    #[ignore_malloc_size_of = "nested Rc"]
    shutdown_action_promise: Rc<RefCell<Option<Rc<Promise>>>>,

    /// The promise resolved or rejected at
    /// <https://streams.spec.whatwg.org/#rs-pipeTo-finalize>
    #[conditional_malloc_size_of]
    result_promise: Rc<Promise>,
}

impl PipeTo {
    /// Run the `abortAlgorithm` defined at
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    pub(crate) fn abort_with_reason(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // Abort should do nothing if we are already shutting down.
        if self.shutting_down.get() {
            return;
        }

        // Let error be signal’s abort reason.
        // Note: storing it because it may need to be kept across a microtask,
        // and see the note below as to why it is kept separately from `shutdown_error`.
        self.abort_reason.set(reason.get());

        // Note: setting the error now,
        // will result in a rejection of the pipe promise, with this error.
        // Unless any shutdown action raise their own error,
        // in which case this error will be overwritten by the shutdown action error.
        self.set_shutdown_error(reason);

        // Let actions be an empty ordered set.
        // Note: the actions are defined, and performed, inside `shutdown_with_an_action`.

        // Shutdown with an action consisting of getting a promise to wait for all of the actions in actions,
        // and with error.
        self.shutdown(cx, global, Some(ShutdownAction::Abort), realm, can_gc);
    }
}

impl Callback for PipeTo {
    /// The pipe makes progress one microtask at a time.
    /// Note: we use one struct as the callback for all promises,
    /// and for both of their reactions.
    ///
    /// The context of the callback is determined from:
    /// - the current state.
    /// - the type of `result`.
    /// - the state of a stored promise(in some cases).
    #[expect(unsafe_code)]
    fn callback(&self, cx: SafeJSContext, result: SafeHandleValue, realm: InRealm, can_gc: CanGc) {
        let global = self.reader.global();

        // Note: we only care about the result of writes when they are rejected,
        // and the error is accessed not through handlers,
        // but directly using `dest.get_stored_error`.
        // So we must mark rejected promises as handled
        // to prevent unhandled rejection errors.
        self.pending_writes.borrow_mut().retain(|p| {
            let pending = p.is_pending();
            if !pending {
                p.set_promise_is_handled();
            }
            pending
        });

        // Note: cloning to prevent re-borrow in methods called below.
        let state_before_checks = self.state.borrow().clone();

        // Note: if we are in a `PendingRead` state,
        // and the source is closed,
        // we try to write chunks before doing any shutdown,
        // which is necessary to implement the
        // "If any chunks have been read but not yet written, write them to dest."
        // part of shutdown.
        if state_before_checks == PipeToState::PendingRead {
            let source = self.reader.get_stream().expect("Source stream must be set");
            if source.is_closed() {
                let dest = self
                    .writer
                    .get_stream()
                    .expect("Destination stream must be set");

                // If dest.[[state]] is "writable",
                // and ! WritableStreamCloseQueuedOrInFlight(dest) is false,
                if dest.is_writable() && !dest.close_queued_or_in_flight() {
                    let Ok(done) = get_read_promise_done(cx, &result, can_gc) else {
                        // This is the case that the microtask ran in reaction
                        // to the closed promise of the reader,
                        // so we should wait for subsequent chunks,
                        // and skip the shutdown below
                        // (reader is closed, but there are still pending reads).
                        // Shutdown will happen when the last chunk has been received.
                        return;
                    };

                    if !done {
                        // If any chunks have been read but not yet written, write them to dest.
                        self.write_chunk(cx, &global, result, can_gc);
                    }
                }
            }
        }

        self.check_and_propagate_errors_forward(cx, &global, realm, can_gc);
        self.check_and_propagate_errors_backward(cx, &global, realm, can_gc);
        self.check_and_propagate_closing_forward(cx, &global, realm, can_gc);
        self.check_and_propagate_closing_backward(cx, &global, realm, can_gc);

        // Note: cloning to prevent re-borrow in methods called below.
        let state = self.state.borrow().clone();

        // If we switched to a shutdown state,
        // return.
        // Progress will be made at the next tick.
        if state != state_before_checks {
            return;
        }

        match state {
            PipeToState::Starting => unreachable!("PipeTo should not be in the Starting state."),
            PipeToState::PendingReady => {
                // Read a chunk.
                self.read_chunk(&global, realm, can_gc);
            },
            PipeToState::PendingRead => {
                // Write the chunk.
                self.write_chunk(cx, &global, result, can_gc);

                // An early return is necessary if the write algorithm aborted the pipe.
                if self.shutting_down.get() {
                    return;
                }

                // Wait for the writer to be ready again.
                self.wait_for_writer_ready(&global, realm, can_gc);
            },
            PipeToState::ShuttingDownWithPendingWrites(action) => {
                // Wait until every chunk that has been read has been written
                // (i.e. the corresponding promises have settled).
                if let Some(write) = self.pending_writes.borrow_mut().front().cloned() {
                    self.wait_on_pending_write(&global, write, realm, can_gc);
                    return;
                }

                // Note: error is stored in `self.shutdown_error`.
                if let Some(action) = action {
                    // Let p be the result of performing action.
                    self.perform_action(cx, &global, action, realm, can_gc);
                } else {
                    // Finalize, passing along error if it was given.
                    self.finalize(cx, &global, can_gc);
                }
            },
            PipeToState::ShuttingDownPendingAction => {
                let Some(ref promise) = *self.shutdown_action_promise.borrow() else {
                    unreachable!();
                };
                if promise.is_pending() {
                    // While waiting for the action to complete,
                    // we may get callbacks for other promises(closed, ready),
                    // and we should ignore those.
                    return;
                }

                let is_array_like = {
                    if !result.is_object() {
                        false
                    } else {
                        unsafe { is_array_like::<crate::DomTypeHolder>(*cx, result) }
                    }
                };

                // Finalize, passing along error if it was given.
                if !result.is_undefined() && !is_array_like {
                    // Most actions either resolve with undefined,
                    // or reject with an error,
                    // and the error should be used when finalizing.
                    // One exception is the `Abort` action,
                    // which resolves with a list of undefined values.

                    // If `result` isn't undefined or array-like,
                    // then it is an error
                    // and should overwrite the current shutdown error.
                    self.set_shutdown_error(result);
                }
                self.finalize(cx, &global, can_gc);
            },
            PipeToState::Finalized => {},
        }
    }
}

impl PipeTo {
    /// Setting shutdown error in a way that ensures it isn't
    /// moved after it has been set.
    fn set_shutdown_error(&self, error: SafeHandleValue) {
        *self.shutdown_error.borrow_mut() = Some(Heap::default());
        let Some(ref heap) = *self.shutdown_error.borrow() else {
            unreachable!("Option set to Some(heap) above.");
        };
        heap.set(error.get())
    }

    /// Wait for the writer to be ready,
    /// which implements the constraint that backpressure must be enforced.
    fn wait_for_writer_ready(&self, global: &GlobalScope, realm: InRealm, can_gc: CanGc) {
        {
            let mut state = self.state.borrow_mut();
            *state = PipeToState::PendingReady;
        }

        let ready_promise = self.writer.Ready();
        if ready_promise.is_fulfilled() {
            self.read_chunk(global, realm, can_gc);
        } else {
            let handler = PromiseNativeHandler::new(
                global,
                Some(Box::new(self.clone())),
                Some(Box::new(self.clone())),
                can_gc,
            );
            ready_promise.append_native_handler(&handler, realm, can_gc);

            // Note: if the writer is not ready,
            // in order to ensure progress we must
            // also react to the closure of the source(because source may close empty).
            let closed_promise = self.reader.Closed();
            closed_promise.append_native_handler(&handler, realm, can_gc);
        }
    }

    /// Read a chunk
    fn read_chunk(&self, global: &GlobalScope, realm: InRealm, can_gc: CanGc) {
        *self.state.borrow_mut() = PipeToState::PendingRead;
        let chunk_promise = self.reader.Read(can_gc);
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(self.clone())),
            Some(Box::new(self.clone())),
            can_gc,
        );
        chunk_promise.append_native_handler(&handler, realm, can_gc);

        // Note: in order to ensure progress we must
        // also react to the closure of the destination.
        let ready_promise = self.writer.Closed();
        ready_promise.append_native_handler(&handler, realm, can_gc);
    }

    /// Try to write a chunk using the jsval, and returns wether it succeeded
    // It will fail if it is the last `done` chunk, or if it is not a chunk at all.
    #[expect(unsafe_code)]
    fn write_chunk(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        chunk: SafeHandleValue,
        can_gc: CanGc,
    ) -> bool {
        if chunk.is_object() {
            rooted!(in(*cx) let object = chunk.to_object());
            rooted!(in(*cx) let mut bytes = UndefinedValue());
            let has_value = unsafe {
                get_dictionary_property(*cx, object.handle(), "value", bytes.handle_mut(), can_gc)
                    .expect("Chunk should have a value.")
            };
            if has_value {
                // Write the chunk.
                let write_promise = self.writer.write(cx, global, bytes.handle(), can_gc);
                self.pending_writes.borrow_mut().push_back(write_promise);
                return true;
            }
        }
        false
    }

    /// Only as part of shutting-down do we wait on pending writes
    /// (backpressure is communicated not through pending writes
    /// but through the readiness of the writer).
    fn wait_on_pending_write(
        &self,
        global: &GlobalScope,
        promise: Rc<Promise>,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(self.clone())),
            Some(Box::new(self.clone())),
            can_gc,
        );
        promise.append_native_handler(&handler, realm, can_gc);
    }

    /// Errors must be propagated forward part of
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    fn check_and_propagate_errors_forward(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // An early return is necessary if we are shutting down,
        // because in that case the source can already have been set to none.
        if self.shutting_down.get() {
            return;
        }

        // if source.[[state]] is or becomes "errored", then
        let source = self
            .reader
            .get_stream()
            .expect("Reader should still have a stream");
        if source.is_errored() {
            rooted!(in(*cx) let mut source_error = UndefinedValue());
            source.get_stored_error(source_error.handle_mut());
            self.set_shutdown_error(source_error.handle());

            // If preventAbort is false,
            if !self.prevent_abort {
                // shutdown with an action of ! WritableStreamAbort(dest, source.[[storedError]])
                // and with source.[[storedError]].
                self.shutdown(
                    cx,
                    global,
                    Some(ShutdownAction::WritableStreamAbort),
                    realm,
                    can_gc,
                )
            } else {
                // Otherwise, shutdown with source.[[storedError]].
                self.shutdown(cx, global, None, realm, can_gc);
            }
        }
    }

    /// Errors must be propagated backward part of
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    fn check_and_propagate_errors_backward(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // An early return is necessary if we are shutting down,
        // because in that case the destination can already have been set to none.
        if self.shutting_down.get() {
            return;
        }

        // if dest.[[state]] is or becomes "errored", then
        let dest = self
            .writer
            .get_stream()
            .expect("Writer should still have a stream");
        if dest.is_errored() {
            rooted!(in(*cx) let mut dest_error = UndefinedValue());
            dest.get_stored_error(dest_error.handle_mut());
            self.set_shutdown_error(dest_error.handle());

            // If preventCancel is false,
            if !self.prevent_cancel {
                // shutdown with an action of ! ReadableStreamCancel(source, dest.[[storedError]])
                // and with dest.[[storedError]].
                self.shutdown(
                    cx,
                    global,
                    Some(ShutdownAction::ReadableStreamCancel),
                    realm,
                    can_gc,
                )
            } else {
                // Otherwise, shutdown with dest.[[storedError]].
                self.shutdown(cx, global, None, realm, can_gc);
            }
        }
    }

    /// Closing must be propagated forward part of
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    fn check_and_propagate_closing_forward(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // An early return is necessary if we are shutting down,
        // because in that case the source can already have been set to none.
        if self.shutting_down.get() {
            return;
        }

        // if source.[[state]] is or becomes "closed", then
        let source = self
            .reader
            .get_stream()
            .expect("Reader should still have a stream");
        if source.is_closed() {
            // If preventClose is false,
            if !self.prevent_close {
                // shutdown with an action of ! WritableStreamAbort(dest, source.[[storedError]])
                // and with source.[[storedError]].
                self.shutdown(
                    cx,
                    global,
                    Some(ShutdownAction::WritableStreamDefaultWriterCloseWithErrorPropagation),
                    realm,
                    can_gc,
                )
            } else {
                // Otherwise, shutdown.
                self.shutdown(cx, global, None, realm, can_gc);
            }
        }
    }

    /// Closing must be propagated backward part of
    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    fn check_and_propagate_closing_backward(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // An early return is necessary if we are shutting down,
        // because in that case the destination can already have been set to none.
        if self.shutting_down.get() {
            return;
        }

        // if ! WritableStreamCloseQueuedOrInFlight(dest) is true
        // or dest.[[state]] is "closed"
        let dest = self
            .writer
            .get_stream()
            .expect("Writer should still have a stream");
        if dest.close_queued_or_in_flight() || dest.is_closed() {
            // Assert: no chunks have been read or written.
            // Note: unclear how to perform this assertion.

            // Let destClosed be a new TypeError.
            rooted!(in(*cx) let mut dest_closed = UndefinedValue());
            let error =
                Error::Type("Destination is closed or has closed queued or in flight".to_string());
            error.to_jsval(cx, global, dest_closed.handle_mut(), can_gc);
            self.set_shutdown_error(dest_closed.handle());

            // If preventCancel is false,
            if !self.prevent_cancel {
                // shutdown with an action of ! ReadableStreamCancel(source, destClosed)
                // and with destClosed.
                self.shutdown(
                    cx,
                    global,
                    Some(ShutdownAction::ReadableStreamCancel),
                    realm,
                    can_gc,
                )
            } else {
                // Otherwise, shutdown with destClosed.
                self.shutdown(cx, global, None, realm, can_gc);
            }
        }
    }

    /// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown-with-action>
    /// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown>
    /// Combined into one method with an optional action.
    fn shutdown(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        action: Option<ShutdownAction>,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        // If shuttingDown is true, abort these substeps.
        // Set shuttingDown to true.
        if !self.shutting_down.replace(true) {
            let dest = self.writer.get_stream().expect("Stream must be set");
            // If dest.[[state]] is "writable",
            // and ! WritableStreamCloseQueuedOrInFlight(dest) is false,
            if dest.is_writable() && !dest.close_queued_or_in_flight() {
                // If any chunks have been read but not yet written, write them to dest.
                // Done at the top of `Callback`.

                // Wait until every chunk that has been read has been written
                // (i.e. the corresponding promises have settled).
                if let Some(write) = self.pending_writes.borrow_mut().front() {
                    *self.state.borrow_mut() = PipeToState::ShuttingDownWithPendingWrites(action);
                    self.wait_on_pending_write(global, write.clone(), realm, can_gc);
                    return;
                }
            }

            // Note: error is stored in `self.shutdown_error`.
            if let Some(action) = action {
                // Let p be the result of performing action.
                self.perform_action(cx, global, action, realm, can_gc);
            } else {
                // Finalize, passing along error if it was given.
                self.finalize(cx, global, can_gc);
            }
        }
    }

    /// The perform action part of
    /// <https://streams.spec.whatwg.org/#rs-pipeTo-shutdown-with-action>
    fn perform_action(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        action: ShutdownAction,
        realm: InRealm,
        can_gc: CanGc,
    ) {
        rooted!(in(*cx) let mut error = UndefinedValue());
        if let Some(shutdown_error) = self.shutdown_error.borrow().as_ref() {
            error.set(shutdown_error.get());
        }

        *self.state.borrow_mut() = PipeToState::ShuttingDownPendingAction;

        // Let p be the result of performing action.
        let promise = match action {
            ShutdownAction::WritableStreamAbort => {
                let dest = self.writer.get_stream().expect("Stream must be set");
                dest.abort(cx, global, error.handle(), realm, can_gc)
            },
            ShutdownAction::ReadableStreamCancel => {
                let source = self
                    .reader
                    .get_stream()
                    .expect("Reader should have a stream.");
                source.cancel(cx, global, error.handle(), can_gc)
            },
            ShutdownAction::WritableStreamDefaultWriterCloseWithErrorPropagation => {
                self.writer.close_with_error_propagation(cx, global, can_gc)
            },
            ShutdownAction::Abort => {
                // Note: implementation of the `abortAlgorithm`
                // of the signal associated with this piping operation.

                // Let error be signal’s abort reason.
                rooted!(in(*cx) let mut error = UndefinedValue());
                error.set(self.abort_reason.get());

                // Let actions be an empty ordered set.
                let mut actions = vec![];

                // If preventAbort is false, append the following action to actions:
                if !self.prevent_abort {
                    let dest = self
                        .writer
                        .get_stream()
                        .expect("Destination stream must be set");

                    // If dest.[[state]] is "writable",
                    let promise = if dest.is_writable() {
                        // return ! WritableStreamAbort(dest, error)
                        dest.abort(cx, global, error.handle(), realm, can_gc)
                    } else {
                        // Otherwise, return a promise resolved with undefined.
                        Promise::new_resolved(global, cx, (), can_gc)
                    };
                    actions.push(promise);
                }

                // If preventCancel is false, append the following action action to actions:
                if !self.prevent_cancel {
                    let source = self.reader.get_stream().expect("Source stream must be set");

                    // If source.[[state]] is "readable",
                    let promise = if source.is_readable() {
                        // return ! ReadableStreamCancel(source, error).
                        source.cancel(cx, global, error.handle(), can_gc)
                    } else {
                        // Otherwise, return a promise resolved with undefined.
                        Promise::new_resolved(global, cx, (), can_gc)
                    };
                    actions.push(promise);
                }

                // Shutdown with an action consisting
                // of getting a promise to wait for all of the actions in actions,
                // and with error.
                wait_for_all_promise(cx, global, actions, realm, can_gc)
            },
        };

        // Upon fulfillment of p, finalize, passing along originalError if it was given.
        // Upon rejection of p with reason newError, finalize with newError.
        let handler = PromiseNativeHandler::new(
            global,
            Some(Box::new(self.clone())),
            Some(Box::new(self.clone())),
            can_gc,
        );
        promise.append_native_handler(&handler, realm, can_gc);
        *self.shutdown_action_promise.borrow_mut() = Some(promise);
    }

    /// <https://streams.spec.whatwg.org/#rs-pipeTo-finalize>
    fn finalize(&self, cx: SafeJSContext, global: &GlobalScope, can_gc: CanGc) {
        *self.state.borrow_mut() = PipeToState::Finalized;

        // Perform ! WritableStreamDefaultWriterRelease(writer).
        self.writer.release(cx, global, can_gc);

        // If reader implements ReadableStreamBYOBReader,
        // perform ! ReadableStreamBYOBReaderRelease(reader).
        // TODO.

        // Otherwise, perform ! ReadableStreamDefaultReaderRelease(reader).
        self.reader
            .release(can_gc)
            .expect("Releasing the reader should not fail");

        // If signal is not undefined, remove abortAlgorithm from signal.
        // Note: since `self.shutdown` is true at this point,
        // the abort algorithm is a no-op,
        // so for now not implementing this step.

        if let Some(shutdown_error) = self.shutdown_error.borrow().as_ref() {
            rooted!(in(*cx) let mut error = UndefinedValue());
            error.set(shutdown_error.get());
            // If error was given, reject promise with error.
            self.result_promise.reject_native(&error.handle(), can_gc);
        } else {
            // Otherwise, resolve promise with undefined.
            self.result_promise.resolve_native(&(), can_gc);
        }
    }
}

/// The fulfillment handler for the reacting to sourceCancelPromise part of
/// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct SourceCancelPromiseFulfillmentHandler {
    #[conditional_malloc_size_of]
    result: Rc<Promise>,
}

impl Callback for SourceCancelPromiseFulfillmentHandler {
    /// The fulfillment handler for the reacting to sourceCancelPromise part of
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, _cx: SafeJSContext, _v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        self.result.resolve_native(&(), can_gc);
    }
}

/// The rejection handler for the reacting to sourceCancelPromise part of
/// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct SourceCancelPromiseRejectionHandler {
    #[conditional_malloc_size_of]
    result: Rc<Promise>,
}

impl Callback for SourceCancelPromiseRejectionHandler {
    /// The rejection handler for the reacting to sourceCancelPromise part of
    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>.
    /// An implementation of <https://webidl.spec.whatwg.org/#dfn-perform-steps-once-promise-is-settled>
    fn callback(&self, _cx: SafeJSContext, v: SafeHandleValue, _realm: InRealm, can_gc: CanGc) {
        self.result.reject_native(&v, can_gc);
    }
}

/// <https://streams.spec.whatwg.org/#readablestream-state>
#[derive(Clone, Copy, Debug, Default, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ReadableStreamState {
    #[default]
    Readable,
    Closed,
    Errored,
}

/// <https://streams.spec.whatwg.org/#readablestream-controller>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ControllerType {
    /// <https://streams.spec.whatwg.org/#readablebytestreamcontroller>
    Byte(MutNullableDom<ReadableByteStreamController>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultcontroller>
    Default(MutNullableDom<ReadableStreamDefaultController>),
}

/// <https://streams.spec.whatwg.org/#readablestream-readerr>
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum ReaderType {
    /// <https://streams.spec.whatwg.org/#readablestreambyobreader>
    #[allow(clippy::upper_case_acronyms)]
    BYOB(MutNullableDom<ReadableStreamBYOBReader>),
    /// <https://streams.spec.whatwg.org/#readablestreamdefaultreader>
    Default(MutNullableDom<ReadableStreamDefaultReader>),
}

impl Eq for ReaderType {}
impl PartialEq for ReaderType {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (ReaderType::BYOB(_), ReaderType::BYOB(_)) |
                (ReaderType::Default(_), ReaderType::Default(_))
        )
    }
}

/// <https://streams.spec.whatwg.org/#create-readable-stream>
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) fn create_readable_stream(
    global: &GlobalScope,
    underlying_source_type: UnderlyingSourceType,
    queuing_strategy: Option<Rc<QueuingStrategySize>>,
    high_water_mark: Option<f64>,
    can_gc: CanGc,
) -> DomRoot<ReadableStream> {
    // If highWaterMark was not passed, set it to 1.
    let high_water_mark = high_water_mark.unwrap_or(1.0);

    // If sizeAlgorithm was not passed, set it to an algorithm that returns 1.
    let size_algorithm =
        queuing_strategy.unwrap_or(extract_size_algorithm(&QueuingStrategy::empty(), can_gc));

    // Assert: ! IsNonNegativeNumber(highWaterMark) is true.
    assert!(high_water_mark >= 0.0);

    // Let stream be a new ReadableStream.
    // Perform ! InitializeReadableStream(stream).
    let stream = ReadableStream::new_with_proto(global, None, can_gc);

    // Let controller be a new ReadableStreamDefaultController.
    let controller = ReadableStreamDefaultController::new(
        global,
        underlying_source_type,
        high_water_mark,
        size_algorithm,
        can_gc,
    );

    // Perform ? SetUpReadableStreamDefaultController(stream, controller, startAlgorithm,
    // pullAlgorithm, cancelAlgorithm, highWaterMark, sizeAlgorithm).
    controller
        .setup(stream.clone(), can_gc)
        .expect("Setup of default controller cannot fail");

    // Return stream.
    stream
}

/// <https://streams.spec.whatwg.org/#abstract-opdef-createreadablebytestream>
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
pub(crate) fn readable_byte_stream_tee(
    global: &GlobalScope,
    underlying_source_type: UnderlyingSourceType,
    can_gc: CanGc,
) -> DomRoot<ReadableStream> {
    // Let stream be a new ReadableStream.
    // Perform ! InitializeReadableStream(stream).
    let tee_stream = ReadableStream::new_with_proto(global, None, can_gc);

    // Let controller be a new ReadableByteStreamController.
    let controller = ReadableByteStreamController::new(underlying_source_type, 0.0, global, can_gc);

    // Perform ? SetUpReadableByteStreamController(stream, controller, startAlgorithm, pullAlgorithm, cancelAlgorithm, 0, undefined).
    controller
        .setup(global, tee_stream.clone(), can_gc)
        .expect("Setup of byte stream controller cannot fail");

    // Return stream.
    tee_stream
}

/// <https://streams.spec.whatwg.org/#rs-class>
#[dom_struct]
pub(crate) struct ReadableStream {
    reflector_: Reflector,

    /// <https://streams.spec.whatwg.org/#readablestream-controller>
    /// Note: the inner `MutNullableDom` should really be an `Option<Dom>`,
    /// because it is never unset once set.
    controller: RefCell<Option<ControllerType>>,

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    #[ignore_malloc_size_of = "mozjs"]
    stored_error: Heap<JSVal>,

    /// <https://streams.spec.whatwg.org/#readablestream-disturbed>
    disturbed: Cell<bool>,

    /// <https://streams.spec.whatwg.org/#readablestream-reader>
    reader: RefCell<Option<ReaderType>>,

    /// <https://streams.spec.whatwg.org/#readablestream-state>
    state: Cell<ReadableStreamState>,
}

impl ReadableStream {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#initialize-readable-stream>
    fn new_inherited() -> ReadableStream {
        ReadableStream {
            reflector_: Reflector::new(),
            controller: RefCell::new(None),
            stored_error: Heap::default(),
            disturbed: Default::default(),
            reader: RefCell::new(None),
            state: Cell::new(Default::default()),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_proto(
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<ReadableStream> {
        reflect_dom_object_with_proto(
            Box::new(ReadableStream::new_inherited()),
            global,
            proto,
            can_gc,
        )
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub(crate) fn set_default_controller(&self, controller: &ReadableStreamDefaultController) {
        *self.controller.borrow_mut() = Some(ControllerType::Default(MutNullableDom::new(Some(
            controller,
        ))));
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller>
    pub(crate) fn set_byte_controller(&self, controller: &ReadableByteStreamController) {
        *self.controller.borrow_mut() =
            Some(ControllerType::Byte(MutNullableDom::new(Some(controller))));
    }

    /// Used as part of
    /// <https://streams.spec.whatwg.org/#set-up-readable-stream-default-controller>
    pub(crate) fn assert_no_controller(&self) {
        let has_no_controller = self.controller.borrow().is_none();
        assert!(has_no_controller);
    }

    /// Build a stream backed by a Rust source that has already been read into memory.
    pub(crate) fn new_from_bytes(
        global: &GlobalScope,
        bytes: Vec<u8>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        let stream = ReadableStream::new_with_external_underlying_source(
            global,
            UnderlyingSourceType::Memory(bytes.len()),
            can_gc,
        )?;
        stream.enqueue_native(bytes, can_gc);
        stream.controller_close_native(can_gc);
        Ok(stream)
    }

    /// Build a stream backed by a Rust underlying source.
    /// Note: external sources are always paired with a default controller.
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_with_external_underlying_source(
        global: &GlobalScope,
        source: UnderlyingSourceType,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        assert!(source.is_native());
        let stream = ReadableStream::new_with_proto(global, None, can_gc);
        let controller = ReadableStreamDefaultController::new(
            global,
            source,
            1.0,
            extract_size_algorithm(&QueuingStrategy::empty(), can_gc),
            can_gc,
        );
        controller.setup(stream.clone(), can_gc)?;
        Ok(stream)
    }

    /// Call into the release steps of the controller,
    pub(crate) fn perform_release_steps(&self) -> Fallible<()> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                let controller = controller
                    .get()
                    .ok_or_else(|| Error::Type("Stream should have controller.".to_string()))?;
                controller.perform_release_steps()
            },
            Some(ControllerType::Byte(controller)) => {
                let controller = controller
                    .get()
                    .ok_or_else(|| Error::Type("Stream should have controller.".to_string()))?;
                controller.perform_release_steps()
            },
            None => Err(Error::Type("Stream should have controller.".to_string())),
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub(crate) fn perform_pull_steps(
        &self,
        cx: SafeJSContext,
        read_request: &ReadRequest,
        can_gc: CanGc,
    ) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_steps(read_request, can_gc),
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_steps(cx, read_request, can_gc),
            None => {
                unreachable!("Stream does not have a controller.");
            },
        }
    }

    /// Call into the pull steps of the controller,
    /// as part of
    /// <https://streams.spec.whatwg.org/#readable-stream-byob-reader-read>
    pub(crate) fn perform_pull_into(
        &self,
        cx: SafeJSContext,
        read_into_request: &ReadIntoRequest,
        view: HeapBufferSource<ArrayBufferViewU8>,
        min: u64,
        can_gc: CanGc,
    ) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_pull_into(cx, read_into_request, view, min, can_gc),
            _ => {
                unreachable!(
                    "Pulling a chunk from a stream with a default controller using a BYOB reader"
                )
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-request>
    pub(crate) fn add_read_request(&self, read_request: &ReadRequest) {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    panic!("Attempt to add a read request without having first acquired a reader.");
                };

                // Assert: stream.[[state]] is "readable".
                assert!(self.is_readable());

                // Append readRequest to stream.[[reader]].[[readRequests]].
                reader.add_read_request(read_request);
            },
            _ => {
                unreachable!("Adding a read request can only be done on a default reader.")
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-add-read-into-request>
    pub(crate) fn add_read_into_request(&self, read_request: &ReadIntoRequest) {
        match self.reader.borrow().as_ref() {
            // Assert: stream.[[reader]] implements ReadableStreamBYOBReader.
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Attempt to add a read into request without having first acquired a reader."
                    );
                };

                // Assert: stream.[[state]] is "readable" or "closed".
                assert!(self.is_readable() || self.is_closed());

                // Append readRequest to stream.[[reader]].[[readIntoRequests]].
                reader.add_read_into_request(read_request);
            },
            _ => {
                unreachable!("Adding a read into request can only be done on a BYOB reader.")
            },
        }
    }

    /// Endpoint to enqueue chunks directly from Rust.
    /// Note: in other use cases this call happens via the controller.
    pub(crate) fn enqueue_native(&self, bytes: Vec<u8>, can_gc: CanGc) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .enqueue_native(bytes, can_gc),
            _ => {
                unreachable!(
                    "Enqueueing chunk to a stream from Rust on other than default controller"
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    pub(crate) fn error(&self, e: SafeHandleValue, can_gc: CanGc) {
        // Assert: stream.[[state]] is "readable".
        assert!(self.is_readable());

        // Set stream.[[state]] to "errored".
        self.state.set(ReadableStreamState::Errored);

        // Set stream.[[storedError]] to e.
        self.stored_error.set(e.get());

        // Let reader be stream.[[reader]].

        let default_reader = {
            let reader_ref = self.reader.borrow();
            match reader_ref.as_ref() {
                Some(ReaderType::Default(reader)) => reader.get(),
                _ => None,
            }
        };

        if let Some(reader) = default_reader {
            // Perform ! ReadableStreamDefaultReaderErrorReadRequests(reader, e).
            reader.error(e, can_gc);
            return;
        }

        let byob_reader = {
            let reader_ref = self.reader.borrow();
            match reader_ref.as_ref() {
                Some(ReaderType::BYOB(reader)) => reader.get(),
                _ => None,
            }
        };

        if let Some(reader) = byob_reader {
            // Perform ! ReadableStreamBYOBReaderErrorReadIntoRequests(reader, e).
            reader.error_read_into_requests(e, can_gc);
        }

        // If reader is undefined, return.
    }

    /// <https://streams.spec.whatwg.org/#readablestream-storederror>
    pub(crate) fn get_stored_error(&self, mut handle_mut: SafeMutableHandleValue) {
        handle_mut.set(self.stored_error.get());
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-error>
    /// Note: in other use cases this call happens via the controller.
    pub(crate) fn error_native(&self, error: Error, can_gc: CanGc) {
        let cx = GlobalScope::get_cx();
        rooted!(in(*cx) let mut error_val = UndefinedValue());
        error.to_jsval(cx, &self.global(), error_val.handle_mut(), can_gc);
        self.error(error_val.handle(), can_gc);
    }

    /// Call into the controller's `Close` method.
    /// <https://streams.spec.whatwg.org/#readable-stream-default-controller-close>
    pub(crate) fn controller_close_native(&self, can_gc: CanGc) {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                let _ = controller
                    .get()
                    .expect("Stream should have controller.")
                    .Close(can_gc);
            },
            _ => {
                unreachable!("Native closing is only done on default controllers.")
            },
        }
    }

    /// Returns a boolean reflecting whether the stream has all data in memory.
    /// Useful for native source integration only.
    pub(crate) fn in_memory(&self) -> bool {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .in_memory(),
            _ => {
                unreachable!(
                    "Checking if source is in memory for a stream with a non-default controller"
                )
            },
        }
    }

    /// Return bytes for synchronous use, if the stream has all data in memory.
    /// Useful for native source integration only.
    pub(crate) fn get_in_memory_bytes(&self) -> Option<GenericSharedMemory> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .get_in_memory_bytes()
                .as_deref()
                .map(GenericSharedMemory::from_bytes),
            _ => {
                unreachable!("Getting in-memory bytes for a stream with a non-default controller")
            },
        }
    }

    /// Acquires a reader and locks the stream,
    /// must be done before `read_a_chunk`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-reader>
    pub(crate) fn acquire_default_reader(
        &self,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStreamDefaultReader>> {
        // Let reader be a new ReadableStreamDefaultReader.
        let reader = ReadableStreamDefaultReader::new(&self.global(), can_gc);

        // Perform ? SetUpReadableStreamDefaultReader(reader, stream).
        reader.set_up(self, &self.global(), can_gc)?;

        // Return reader.
        Ok(reader)
    }

    /// <https://streams.spec.whatwg.org/#acquire-readable-stream-byob-reader>
    pub(crate) fn acquire_byob_reader(
        &self,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStreamBYOBReader>> {
        // Let reader be a new ReadableStreamBYOBReader.
        let reader = ReadableStreamBYOBReader::new(&self.global(), can_gc);
        // Perform ? SetUpReadableStreamBYOBReader(reader, stream).
        reader.set_up(self, &self.global(), can_gc)?;

        // Return reader.
        Ok(reader)
    }

    pub(crate) fn get_default_controller(&self) -> DomRoot<ReadableStreamDefaultController> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => {
                controller.get().expect("Stream should have controller.")
            },
            _ => {
                unreachable!(
                    "Getting default controller for a stream with a non-default controller"
                )
            },
        }
    }

    pub(crate) fn get_byte_controller(&self) -> DomRoot<ReadableByteStreamController> {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Byte(controller)) => {
                controller.get().expect("Stream should have controller.")
            },
            _ => {
                unreachable!("Getting byte controller for a stream with a non-byte controller")
            },
        }
    }

    pub(crate) fn get_default_reader(&self) -> DomRoot<ReadableStreamDefaultReader> {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().expect("Stream should have reader."),
            _ => {
                unreachable!("Getting default reader for a stream with a non-default reader")
            },
        }
    }

    /// Read a chunk from the stream,
    /// must be called after `start_reading`,
    /// and before `stop_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#readable-stream-default-reader-read>
    pub(crate) fn read_a_chunk(&self, can_gc: CanGc) -> Rc<Promise> {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Attempt to read stream chunk without having first acquired a reader."
                    );
                };
                reader.Read(can_gc)
            },
            _ => {
                unreachable!("Native reading of a chunk can only be done with a default reader.")
            },
        }
    }

    /// Releases the lock on the reader,
    /// must be done after `start_reading`.
    /// Native call to
    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaultreaderrelease>
    pub(crate) fn stop_reading(&self, can_gc: CanGc) {
        let reader_ref = self.reader.borrow();

        match reader_ref.as_ref() {
            Some(ReaderType::Default(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!("Attempt to stop reading without having first acquired a reader.");
                };

                drop(reader_ref);
                reader.release(can_gc).expect("Reader release cannot fail.");
            },
            _ => {
                unreachable!("Native stop reading can only be done with a default reader.")
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#is-readable-stream-locked>
    pub(crate) fn is_locked(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().is_some(),
            Some(ReaderType::BYOB(reader)) => reader.get().is_some(),
            None => false,
        }
    }

    pub(crate) fn is_disturbed(&self) -> bool {
        self.disturbed.get()
    }

    pub(crate) fn set_is_disturbed(&self, disturbed: bool) {
        self.disturbed.set(disturbed);
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.state.get() == ReadableStreamState::Closed
    }

    pub(crate) fn is_errored(&self) -> bool {
        self.state.get() == ReadableStreamState::Errored
    }

    pub(crate) fn is_readable(&self) -> bool {
        self.state.get() == ReadableStreamState::Readable
    }

    pub(crate) fn has_default_reader(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => reader.get().is_some(),
            _ => false,
        }
    }

    pub(crate) fn has_byob_reader(&self) -> bool {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => reader.get().is_some(),
            _ => false,
        }
    }

    pub(crate) fn has_byte_controller(&self) -> bool {
        match self.controller.borrow().as_ref() {
            Some(ControllerType::Byte(controller)) => controller.get().is_some(),
            _ => false,
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-requests>
    pub(crate) fn get_num_read_requests(&self) -> usize {
        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when getting the number of read requests.");
                reader.get_num_read_requests()
            },
            _ => unreachable!(
                "Stream must have a default reader when get num read requests is called into."
            ),
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-get-num-read-into-requests>
    pub(crate) fn get_num_read_into_requests(&self) -> usize {
        assert!(self.has_byob_reader());

        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Stream must have a reader when get num read into requests is called into."
                    );
                };
                reader.get_num_read_into_requests()
            },
            _ => {
                unreachable!(
                    "Stream must have a BYOB reader when get num read into requests is called into."
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-request>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn fulfill_read_request(&self, chunk: SafeHandleValue, done: bool, can_gc: CanGc) {
        // step 1 - Assert: ! ReadableStreamHasDefaultReader(stream) is true.
        assert!(self.has_default_reader());

        match self.reader.borrow().as_ref() {
            Some(ReaderType::Default(reader)) => {
                // step 2 - Let reader be stream.[[reader]].
                let reader = reader
                    .get()
                    .expect("Stream must have a reader when a read request is fulfilled.");
                // step 3 - Assert: reader.[[readRequests]] is not empty.
                assert_ne!(reader.get_num_read_requests(), 0);
                // step 4 & 5
                // Let readRequest be reader.[[readRequests]][0]. & Remove readRequest from reader.[[readRequests]].
                let request = reader.remove_read_request();

                if done {
                    // step 6 - If done is true, perform readRequest’s close steps.
                    request.close_steps(can_gc);
                } else {
                    // step 7 - Otherwise, perform readRequest’s chunk steps, given chunk.
                    let result = RootedTraceableBox::new(Heap::default());
                    result.set(*chunk);
                    request.chunk_steps(result, &self.global(), can_gc);
                }
            },
            _ => {
                unreachable!(
                    "Stream must have a default reader when fulfill read requests is called into."
                );
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-fulfill-read-into-request>
    pub(crate) fn fulfill_read_into_request(
        &self,
        chunk: SafeHandleValue,
        done: bool,
        can_gc: CanGc,
    ) {
        // Assert: ! ReadableStreamHasBYOBReader(stream) is true.
        assert!(self.has_byob_reader());

        // Let reader be stream.[[reader]].
        match self.reader.borrow().as_ref() {
            Some(ReaderType::BYOB(reader)) => {
                let Some(reader) = reader.get() else {
                    unreachable!(
                        "Stream must have a reader when a read into request is fulfilled."
                    );
                };

                // Assert: reader.[[readIntoRequests]] is not empty.
                assert!(reader.get_num_read_into_requests() > 0);

                // Let readIntoRequest be reader.[[readIntoRequests]][0].
                // Remove readIntoRequest from reader.[[readIntoRequests]].
                let read_into_request = reader.remove_read_into_request();

                // If done is true, perform readIntoRequest’s close steps, given chunk.
                let result = RootedTraceableBox::new(Heap::default());
                if done {
                    result.set(*chunk);
                    read_into_request.close_steps(Some(result), can_gc);
                } else {
                    // Otherwise, perform readIntoRequest’s chunk steps, given chunk.
                    result.set(*chunk);
                    read_into_request.chunk_steps(result, can_gc);
                }
            },
            _ => {
                unreachable!(
                    "Stream must have a BYOB reader when fulfill read into requests is called into."
                );
            },
        };
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-close>
    pub(crate) fn close(&self, can_gc: CanGc) {
        // Assert: stream.[[state]] is "readable".
        assert!(self.is_readable());
        // Set stream.[[state]] to "closed".
        self.state.set(ReadableStreamState::Closed);
        // Let reader be stream.[[reader]].

        // NOTE: do not hold the RefCell borrow across reader.close(),
        // or release() will panic when it tries to mut-borrow stream.reader.
        // So we pull out the underlying DOM reader in a local, then drop the borrow.
        let default_reader = {
            let reader_ref = self.reader.borrow();
            match reader_ref.as_ref() {
                Some(ReaderType::Default(reader)) => reader.get(),
                _ => None,
            }
        };

        if let Some(reader) = default_reader {
            // steps 5 & 6 for a default reader
            reader.close(can_gc);
            return;
        }

        // Same for BYOB reader.
        let byob_reader = {
            let reader_ref = self.reader.borrow();
            match reader_ref.as_ref() {
                Some(ReaderType::BYOB(reader)) => reader.get(),
                _ => None,
            }
        };

        if let Some(reader) = byob_reader {
            // steps 5 & 6 for a BYOB reader
            reader.close(can_gc);
        }

        // If reader is undefined, return.
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-cancel>
    pub(crate) fn cancel(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        reason: SafeHandleValue,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Set stream.[[disturbed]] to true.
        self.disturbed.set(true);

        // If stream.[[state]] is "closed", return a promise resolved with undefined.
        if self.is_closed() {
            return Promise::new_resolved(global, cx, (), can_gc);
        }
        // If stream.[[state]] is "errored", return a promise rejected with stream.[[storedError]].
        if self.is_errored() {
            let promise = Promise::new(global, can_gc);
            rooted!(in(*cx) let mut rval = UndefinedValue());
            self.stored_error
                .safe_to_jsval(cx, rval.handle_mut(), can_gc);
            promise.reject_native(&rval.handle(), can_gc);
            return promise;
        }
        // Perform ! ReadableStreamClose(stream).
        self.close(can_gc);

        // If reader is not undefined and reader implements ReadableStreamBYOBReader,
        let byob_reader = {
            let reader_ref = self.reader.borrow();
            match reader_ref.as_ref() {
                Some(ReaderType::BYOB(reader)) => reader.get(),
                _ => None,
            }
        };

        if let Some(reader) = byob_reader {
            // step 6.1, 6.2 & 6.3 of https://streams.spec.whatwg.org/#readable-stream-cancel
            reader.cancel(can_gc);
        }

        // Let sourceCancelPromise be ! stream.[[controller]].[[CancelSteps]](reason).

        let source_cancel_promise = match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_cancel_steps(cx, global, reason, can_gc),
            Some(ControllerType::Byte(controller)) => controller
                .get()
                .expect("Stream should have controller.")
                .perform_cancel_steps(cx, global, reason, can_gc),
            None => {
                panic!("Stream does not have a controller.");
            },
        };

        // Create a new promise,
        // and setup a handler in order to react to the fulfillment of sourceCancelPromise.
        let global = self.global();
        let result_promise = Promise::new(&global, can_gc);
        let fulfillment_handler = Box::new(SourceCancelPromiseFulfillmentHandler {
            result: result_promise.clone(),
        });
        let rejection_handler = Box::new(SourceCancelPromiseRejectionHandler {
            result: result_promise.clone(),
        });
        let handler = PromiseNativeHandler::new(
            &global,
            Some(fulfillment_handler),
            Some(rejection_handler),
            can_gc,
        );
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        source_cancel_promise.append_native_handler(&handler, comp, can_gc);

        // Return the result of reacting to sourceCancelPromise
        // with a fulfillment step that returns undefined.
        result_promise
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_reader(&self, new_reader: Option<ReaderType>) {
        *self.reader.borrow_mut() = new_reader;
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablebytestreamtee>
    fn byte_tee(&self, can_gc: CanGc) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Assert: stream implements ReadableStream.
        // Assert: stream.[[controller]] implements ReadableByteStreamController.

        // Let reader be ? AcquireReadableStreamDefaultReader(stream).
        let reader = self.acquire_default_reader(can_gc)?;
        let reader = Rc::new(RefCell::new(ReaderType::Default(MutNullableDom::new(
            Some(&reader),
        ))));

        // Let reading be false.
        let reading = Rc::new(Cell::new(false));

        // Let readAgainForBranch1 be false.
        let read_again_for_branch_1 = Rc::new(Cell::new(false));

        // Let readAgainForBranch2 be false.
        let read_again_for_branch_2 = Rc::new(Cell::new(false));

        // Let canceled1 be false.
        let canceled_1 = Rc::new(Cell::new(false));

        // Let canceled2 be false.
        let canceled_2 = Rc::new(Cell::new(false));

        // Let reason1 be undefined.
        let reason_1 = Rc::new(Heap::boxed(UndefinedValue()));

        // Let reason2 be undefined.
        let reason_2 = Rc::new(Heap::boxed(UndefinedValue()));

        // Let cancelPromise be a new promise.
        let cancel_promise = Promise::new(&self.global(), can_gc);
        let reader_version = Rc::new(Cell::new(0));

        let byte_tee_source_1 = ByteTeeUnderlyingSource::new(
            reader.clone(),
            self,
            reading.clone(),
            read_again_for_branch_1.clone(),
            read_again_for_branch_2.clone(),
            canceled_1.clone(),
            canceled_2.clone(),
            reason_1.clone(),
            reason_2.clone(),
            cancel_promise.clone(),
            reader_version.clone(),
            ByteTeeCancelAlgorithm::Cancel1Algorithm,
            ByteTeePullAlgorithm::Pull1Algorithm,
            can_gc,
        );

        let byte_tee_source_2 = ByteTeeUnderlyingSource::new(
            reader.clone(),
            self,
            reading,
            read_again_for_branch_1,
            read_again_for_branch_2,
            canceled_1,
            canceled_2,
            reason_1,
            reason_2,
            cancel_promise.clone(),
            reader_version,
            ByteTeeCancelAlgorithm::Cancel2Algorithm,
            ByteTeePullAlgorithm::Pull2Algorithm,
            can_gc,
        );

        // Set branch1 to ! CreateReadableByteStream(startAlgorithm, pull1Algorithm, cancel1Algorithm).
        let branch_1 = readable_byte_stream_tee(
            &self.global(),
            UnderlyingSourceType::TeeByte(Dom::from_ref(&byte_tee_source_1)),
            can_gc,
        );
        byte_tee_source_1.set_branch_1(&branch_1);
        byte_tee_source_2.set_branch_1(&branch_1);

        // Set branch2 to ! CreateReadableByteStream(startAlgorithm, pull2Algorithm, cancel2Algorithm).
        let branch_2 = readable_byte_stream_tee(
            &self.global(),
            UnderlyingSourceType::TeeByte(Dom::from_ref(&byte_tee_source_2)),
            can_gc,
        );
        byte_tee_source_1.set_branch_2(&branch_2);
        byte_tee_source_2.set_branch_2(&branch_2);

        // Perform forwardReaderError, given reader.
        byte_tee_source_1.forward_reader_error(reader.clone(), can_gc);
        byte_tee_source_2.forward_reader_error(reader, can_gc);

        // Return « branch1, branch2 ».
        Ok(vec![branch_1, branch_2])
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-readablestreamdefaulttee>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn default_tee(
        &self,
        clone_for_branch_2: bool,
        can_gc: CanGc,
    ) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Assert: stream implements ReadableStream.

        // Assert: cloneForBranch2 is a boolean.
        let clone_for_branch_2 = Rc::new(Cell::new(clone_for_branch_2));

        // Let reader be ? AcquireReadableStreamDefaultReader(stream).
        let reader = self.acquire_default_reader(can_gc)?;

        // Let reading be false.
        let reading = Rc::new(Cell::new(false));
        // Let readAgain be false.
        let read_again = Rc::new(Cell::new(false));
        // Let canceled1 be false.
        let canceled_1 = Rc::new(Cell::new(false));
        // Let canceled2 be false.
        let canceled_2 = Rc::new(Cell::new(false));

        // Let reason1 be undefined.
        let reason_1 = Rc::new(Heap::boxed(UndefinedValue()));
        // Let reason2 be undefined.
        let reason_2 = Rc::new(Heap::boxed(UndefinedValue()));
        // Let cancelPromise be a new promise.
        let cancel_promise = Promise::new(&self.global(), can_gc);

        let tee_source_1 = DefaultTeeUnderlyingSource::new(
            &reader,
            self,
            reading.clone(),
            read_again.clone(),
            canceled_1.clone(),
            canceled_2.clone(),
            clone_for_branch_2.clone(),
            reason_1.clone(),
            reason_2.clone(),
            cancel_promise.clone(),
            DefaultTeeCancelAlgorithm::Cancel1Algorithm,
            can_gc,
        );

        let underlying_source_type_branch_1 =
            UnderlyingSourceType::Tee(Dom::from_ref(&tee_source_1));

        let tee_source_2 = DefaultTeeUnderlyingSource::new(
            &reader,
            self,
            reading,
            read_again,
            canceled_1.clone(),
            canceled_2.clone(),
            clone_for_branch_2,
            reason_1,
            reason_2,
            cancel_promise.clone(),
            DefaultTeeCancelAlgorithm::Cancel2Algorithm,
            can_gc,
        );

        let underlying_source_type_branch_2 =
            UnderlyingSourceType::Tee(Dom::from_ref(&tee_source_2));

        // Set branch_1 to ! CreateReadableStream(startAlgorithm, pullAlgorithm, cancel1Algorithm).
        let branch_1 = create_readable_stream(
            &self.global(),
            underlying_source_type_branch_1,
            None,
            None,
            can_gc,
        );
        tee_source_1.set_branch_1(&branch_1);
        tee_source_2.set_branch_1(&branch_1);

        // Set branch_2 to ! CreateReadableStream(startAlgorithm, pullAlgorithm, cancel2Algorithm).
        let branch_2 = create_readable_stream(
            &self.global(),
            underlying_source_type_branch_2,
            None,
            None,
            can_gc,
        );
        tee_source_1.set_branch_2(&branch_2);
        tee_source_2.set_branch_2(&branch_2);

        // Upon rejection of reader.[[closedPromise]] with reason r,
        reader.default_tee_append_native_handler_to_closed_promise(
            &branch_1,
            &branch_2,
            canceled_1,
            canceled_2,
            cancel_promise,
            can_gc,
        );

        // Return « branch_1, branch_2 ».
        Ok(vec![branch_1, branch_2])
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-pipe-to>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn pipe_to(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        dest: &WritableStream,
        prevent_close: bool,
        prevent_abort: bool,
        prevent_cancel: bool,
        signal: Option<&AbortSignal>,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // Assert: source implements ReadableStream.
        // Assert: dest implements WritableStream.
        // Assert: prevent_close, prevent_abort, and prevent_cancel are all booleans.
        // Done with method signature types.

        // If signal was not given, let signal be undefined.
        // Assert: either signal is undefined, or signal implements AbortSignal.
        // Note: done with the `signal` argument.

        // Assert: ! IsReadableStreamLocked(source) is false.
        assert!(!self.is_locked());

        // Assert: ! IsWritableStreamLocked(dest) is false.
        assert!(!dest.is_locked());

        // If source.[[controller]] implements ReadableByteStreamController,
        // let reader be either ! AcquireReadableStreamBYOBReader(source)
        // or ! AcquireReadableStreamDefaultReader(source),
        // at the user agent’s discretion.
        // Note: for now only using default readers.

        // Otherwise, let reader be ! AcquireReadableStreamDefaultReader(source).
        let reader = self
            .acquire_default_reader(can_gc)
            .expect("Acquiring a default reader for pipe_to cannot fail");

        // Let writer be ! AcquireWritableStreamDefaultWriter(dest).
        let writer = dest
            .aquire_default_writer(cx, global, can_gc)
            .expect("Acquiring a default writer for pipe_to cannot fail");

        // Set source.[[disturbed]] to true.
        self.disturbed.set(true);

        // Let shuttingDown be false.
        // Done below with default.

        // Let promise be a new promise.
        let promise = Promise::new(global, can_gc);

        // In parallel, but not really, using reader and writer, read all chunks from source and write them to dest.
        rooted!(in(*cx) let pipe_to = PipeTo {
            reader: Dom::from_ref(&reader),
            writer: Dom::from_ref(&writer),
            pending_writes: Default::default(),
            state: Default::default(),
            prevent_abort,
            prevent_cancel,
            prevent_close,
            shutting_down: Default::default(),
            abort_reason: Default::default(),
            shutdown_error: Default::default(),
            shutdown_action_promise:  Default::default(),
            result_promise: promise.clone(),
        });

        // If signal is not undefined,
        // Note: moving the steps to here, so that the `PipeTo` is available.
        if let Some(signal) = signal {
            // Let abortAlgorithm be the following steps:
            // Note: steps are implemented at call site.
            rooted!(in(*cx) let abort_algorithm = AbortAlgorithm::StreamPiping(pipe_to.clone()));

            // If signal is aborted, perform abortAlgorithm and return promise.
            if signal.aborted() {
                signal.run_abort_algorithm(cx, global, &abort_algorithm, realm, can_gc);
                return promise;
            }

            // Add abortAlgorithm to signal.
            signal.add(&abort_algorithm);
        }

        // Note: perfom checks now, since streams can start as closed or errored.
        pipe_to.check_and_propagate_errors_forward(cx, global, realm, can_gc);
        pipe_to.check_and_propagate_errors_backward(cx, global, realm, can_gc);
        pipe_to.check_and_propagate_closing_forward(cx, global, realm, can_gc);
        pipe_to.check_and_propagate_closing_backward(cx, global, realm, can_gc);

        // If we are not closed or errored,
        if *pipe_to.state.borrow() == PipeToState::Starting {
            // Start the pipe, by waiting on the writer being ready for a chunk.
            pipe_to.wait_for_writer_ready(global, realm, can_gc);
        }

        // Return promise.
        promise
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-tee>
    fn tee(
        &self,
        clone_for_branch_2: bool,
        can_gc: CanGc,
    ) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Assert: stream implements ReadableStream.
        // Assert: cloneForBranch2 is a boolean.

        match self.controller.borrow().as_ref() {
            Some(ControllerType::Default(_)) => {
                // Return ? ReadableStreamDefaultTee(stream, cloneForBranch2).
                self.default_tee(clone_for_branch_2, can_gc)
            },
            Some(ControllerType::Byte(_)) => {
                // If stream.[[controller]] implements ReadableByteStreamController,
                // return ? ReadableByteStreamTee(stream).
                self.byte_tee(can_gc)
            },
            None => {
                unreachable!("Stream should have a controller.");
            },
        }
    }

    /// <https://streams.spec.whatwg.org/#set-up-readable-byte-stream-controller-from-underlying-source>
    pub(crate) fn set_up_byte_controller(
        &self,
        global: &GlobalScope,
        underlying_source_dict: JsUnderlyingSource,
        underlying_source_handle: SafeHandleObject,
        stream: DomRoot<ReadableStream>,
        strategy_hwm: f64,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Let pullAlgorithm be an algorithm that returns a promise resolved with undefined.
        // Let cancelAlgorithm be an algorithm that returns a promise resolved with undefined.
        // If underlyingSourceDict["start"] exists, then set startAlgorithm to an algorithm which returns the result
        // of invoking underlyingSourceDict["start"] with argument list « controller »
        // and callback this value underlyingSource.
        // If underlyingSourceDict["pull"] exists, then set pullAlgorithm to an algorithm which returns the result
        // of invoking underlyingSourceDict["pull"] with argument list « controller »
        // and callback this value underlyingSource.
        // If underlyingSourceDict["cancel"] exists, then set cancelAlgorithm to an algorithm which takes an
        // argument reason and returns the result of invoking underlyingSourceDict["cancel"] with argument list
        // « reason » and callback this value underlyingSource.

        // Let autoAllocateChunkSize be underlyingSourceDict["autoAllocateChunkSize"],
        // if it exists, or undefined otherwise.
        // If autoAllocateChunkSize is 0, then throw a TypeError exception.
        if let Some(0) = underlying_source_dict.autoAllocateChunkSize {
            return Err(Error::Type("autoAllocateChunkSize cannot be 0".to_owned()));
        }

        let controller = ReadableByteStreamController::new(
            UnderlyingSourceType::Js(underlying_source_dict, Heap::default()),
            strategy_hwm,
            global,
            can_gc,
        );

        // Note: this must be done before `setup`,
        // otherwise `thisOb` is null in the start callback.
        controller.set_underlying_source_this_object(underlying_source_handle);

        // Perform ? SetUpReadableByteStreamController(stream, controller, startAlgorithm,
        // pullAlgorithm, cancelAlgorithm, highWaterMark, autoAllocateChunkSize).
        controller.setup(global, stream, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
    pub(crate) fn setup_cross_realm_transform_readable(
        &self,
        cx: SafeJSContext,
        port: &MessagePort,
        can_gc: CanGc,
    ) {
        let port_id = port.message_port_id();
        let global = self.global();

        // Perform ! InitializeReadableStream(stream).
        // Done in `new_inherited`.

        // Let sizeAlgorithm be an algorithm that returns 1.
        let size_algorithm = extract_size_algorithm(&QueuingStrategy::default(), can_gc);

        // Note: other algorithms defined in the underlying source container.

        // Let controller be a new ReadableStreamDefaultController.
        let controller = ReadableStreamDefaultController::new(
            &self.global(),
            UnderlyingSourceType::Transfer(Dom::from_ref(port)),
            0.,
            size_algorithm,
            can_gc,
        );

        // Add a handler for port’s message event with the following steps:
        // Add a handler for port’s messageerror event with the following steps:
        rooted!(in(*cx) let cross_realm_transform_readable = CrossRealmTransformReadable {
            controller: Dom::from_ref(&controller),
        });
        global.note_cross_realm_transform_readable(&cross_realm_transform_readable, port_id);

        // Enable port’s port message queue.
        port.Start(can_gc);

        // Perform ! SetUpReadableStreamDefaultController
        controller
            .setup(DomRoot::from_ref(self), can_gc)
            .expect("Setting up controller for transfer cannot fail.");
    }
}

impl ReadableStreamMethods<crate::DomTypeHolder> for ReadableStream {
    /// <https://streams.spec.whatwg.org/#rs-constructor>
    fn Constructor(
        cx: SafeJSContext,
        global: &GlobalScope,
        proto: Option<SafeHandleObject>,
        can_gc: CanGc,
        underlying_source: Option<*mut JSObject>,
        strategy: &QueuingStrategy,
    ) -> Fallible<DomRoot<Self>> {
        // If underlyingSource is missing, set it to null.
        rooted!(in(*cx) let underlying_source_obj = underlying_source.unwrap_or(ptr::null_mut()));
        // Let underlyingSourceDict be underlyingSource,
        // converted to an IDL value of type UnderlyingSource.
        let underlying_source_dict = if !underlying_source_obj.is_null() {
            rooted!(in(*cx) let obj_val = ObjectValue(underlying_source_obj.get()));
            match JsUnderlyingSource::new(cx, obj_val.handle(), can_gc) {
                Ok(ConversionResult::Success(val)) => val,
                Ok(ConversionResult::Failure(error)) => return Err(Error::Type(error.to_string())),
                _ => {
                    return Err(Error::JSFailed);
                },
            }
        } else {
            JsUnderlyingSource::empty()
        };

        // Perform ! InitializeReadableStream(this).
        let stream = ReadableStream::new_with_proto(global, proto, can_gc);

        if underlying_source_dict.type_.is_some() {
            // If strategy["size"] exists, throw a RangeError exception.
            if strategy.size.is_some() {
                return Err(Error::Range(
                    "size is not supported for byte streams".to_owned(),
                ));
            }

            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 0).
            let strategy_hwm = extract_high_water_mark(strategy, 0.0)?;

            // Perform ? SetUpReadableByteStreamControllerFromUnderlyingSource(this,
            // underlyingSource, underlyingSourceDict, highWaterMark).
            stream.set_up_byte_controller(
                global,
                underlying_source_dict,
                underlying_source_obj.handle(),
                stream.clone(),
                strategy_hwm,
                can_gc,
            )?;
        } else {
            // Let highWaterMark be ? ExtractHighWaterMark(strategy, 1).
            let high_water_mark = extract_high_water_mark(strategy, 1.0)?;

            // Let sizeAlgorithm be ! ExtractSizeAlgorithm(strategy).
            let size_algorithm = extract_size_algorithm(strategy, can_gc);

            let controller = ReadableStreamDefaultController::new(
                global,
                UnderlyingSourceType::Js(underlying_source_dict, Heap::default()),
                high_water_mark,
                size_algorithm,
                can_gc,
            );

            // Note: this must be done before `setup`,
            // otherwise `thisOb` is null in the start callback.
            controller.set_underlying_source_this_object(underlying_source_obj.handle());

            // Perform ? SetUpReadableStreamDefaultControllerFromUnderlyingSource
            controller.setup(stream.clone(), can_gc)?;
        };

        Ok(stream)
    }

    /// <https://streams.spec.whatwg.org/#rs-locked>
    fn Locked(&self) -> bool {
        self.is_locked()
    }

    /// <https://streams.spec.whatwg.org/#rs-cancel>
    fn Cancel(&self, cx: SafeJSContext, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        let global = self.global();
        if self.is_locked() {
            // If ! IsReadableStreamLocked(this) is true,
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("stream is locked".to_owned()), can_gc);
            promise
        } else {
            // Return ! ReadableStreamCancel(this, reason).
            self.cancel(cx, &global, reason, can_gc)
        }
    }

    /// <https://streams.spec.whatwg.org/#rs-get-reader>
    fn GetReader(
        &self,
        options: &ReadableStreamGetReaderOptions,
        can_gc: CanGc,
    ) -> Fallible<ReadableStreamReader> {
        // 1, If options["mode"] does not exist, return ? AcquireReadableStreamDefaultReader(this).
        if options.mode.is_none() {
            return Ok(ReadableStreamReader::ReadableStreamDefaultReader(
                self.acquire_default_reader(can_gc)?,
            ));
        }
        // 2. Assert: options["mode"] is "byob".
        assert!(options.mode.unwrap() == ReadableStreamReaderMode::Byob);

        // 3. Return ? AcquireReadableStreamBYOBReader(this).
        Ok(ReadableStreamReader::ReadableStreamBYOBReader(
            self.acquire_byob_reader(can_gc)?,
        ))
    }

    /// <https://streams.spec.whatwg.org/#rs-tee>
    fn Tee(&self, can_gc: CanGc) -> Fallible<Vec<DomRoot<ReadableStream>>> {
        // Return ? ReadableStreamTee(this, false).
        self.tee(false, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#rs-pipe-to>
    fn PipeTo(
        &self,
        destination: &WritableStream,
        options: &StreamPipeOptions,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        let cx = GlobalScope::get_cx();
        let global = self.global();

        // If ! IsReadableStreamLocked(this) is true,
        if self.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(Error::Type("Source stream is locked".to_owned()), can_gc);
            return promise;
        }

        // If ! IsWritableStreamLocked(destination) is true,
        if destination.is_locked() {
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(&global, can_gc);
            promise.reject_error(
                Error::Type("Destination stream is locked".to_owned()),
                can_gc,
            );
            return promise;
        }

        // Let signal be options["signal"] if it exists, or undefined otherwise.
        let signal = options.signal.as_deref();

        // Return ! ReadableStreamPipeTo.
        self.pipe_to(
            cx,
            &global,
            destination,
            options.preventClose,
            options.preventAbort,
            options.preventCancel,
            signal,
            realm,
            can_gc,
        )
    }

    /// <https://streams.spec.whatwg.org/#rs-pipe-through>
    fn PipeThrough(
        &self,
        transform: &ReadableWritablePair,
        options: &StreamPipeOptions,
        realm: InRealm,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ReadableStream>> {
        let global = self.global();
        let cx = GlobalScope::get_cx();

        // If ! IsReadableStreamLocked(this) is true, throw a TypeError exception.
        if self.is_locked() {
            return Err(Error::Type("Source stream is locked".to_owned()));
        }

        // If ! IsWritableStreamLocked(transform["writable"]) is true, throw a TypeError exception.
        if transform.writable.is_locked() {
            return Err(Error::Type("Destination stream is locked".to_owned()));
        }

        // Let signal be options["signal"] if it exists, or undefined otherwise.
        let signal = options.signal.as_deref();

        // Let promise be ! ReadableStreamPipeTo(this, transform["writable"],
        // options["preventClose"], options["preventAbort"], options["preventCancel"], signal).
        let promise = self.pipe_to(
            cx,
            &global,
            &transform.writable,
            options.preventClose,
            options.preventAbort,
            options.preventCancel,
            signal,
            realm,
            can_gc,
        );

        // Set promise.[[PromiseIsHandled]] to true.
        promise.set_promise_is_handled();

        // Return transform["readable"].
        Ok(transform.readable.clone())
    }
}

#[expect(unsafe_code)]
/// The initial steps for the message handler for both readable and writable cross realm transforms.
/// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
/// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
pub(crate) unsafe fn get_type_and_value_from_message(
    cx: SafeJSContext,
    data: SafeHandleValue,
    value: SafeMutableHandleValue,
    can_gc: CanGc,
) -> DOMString {
    // Let data be the data of the message.
    // Note: we are passed the data as argument,
    // which originates in the return value of `structuredclone::read`.

    // Assert: data is an Object.
    assert!(data.is_object());
    rooted!(in(*cx) let data_object = data.to_object());

    // Let type be ! Get(data, "type").
    rooted!(in(*cx) let mut type_ = UndefinedValue());
    unsafe {
        get_dictionary_property(
            *cx,
            data_object.handle(),
            "type",
            type_.handle_mut(),
            can_gc,
        )
    }
    .expect("Getting the type should not fail.");

    // Let value be ! Get(data, "value").
    unsafe { get_dictionary_property(*cx, data_object.handle(), "value", value, can_gc) }
        .expect("Getting the value should not fail.");

    // Assert: type is a String.
    let result =
        DOMString::safe_from_jsval(cx, type_.handle(), StringificationBehavior::Empty, can_gc)
            .expect("The type of the message should be a string");
    let ConversionResult::Success(type_string) = result else {
        unreachable!("The type of the message should be a string");
    };

    type_string
}

impl js::gc::Rootable for CrossRealmTransformReadable {}

/// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
/// A wrapper to handle `message` and `messageerror` events
/// for the port used by the transfered stream.
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct CrossRealmTransformReadable {
    /// The controller used in the algorithm.
    controller: Dom<ReadableStreamDefaultController>,
}

impl CrossRealmTransformReadable {
    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformreadable>
    /// Add a handler for port’s message event with the following steps:
    #[expect(unsafe_code)]
    pub(crate) fn handle_message(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        port: &MessagePort,
        message: SafeHandleValue,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        rooted!(in(*cx) let mut value = UndefinedValue());
        let type_string =
            unsafe { get_type_and_value_from_message(cx, message, value.handle_mut(), can_gc) };

        // If type is "chunk",
        if type_string == "chunk" {
            // Perform ! ReadableStreamDefaultControllerEnqueue(controller, value).
            self.controller
                .enqueue(cx, value.handle(), can_gc)
                .expect("Enqueing a chunk should not fail.");
        }

        // Otherwise, if type is "close",
        if type_string == "close" {
            // Perform ! ReadableStreamDefaultControllerClose(controller).
            self.controller.close(can_gc);

            // Disentangle port.
            global.disentangle_port(port, can_gc);
        }

        // Otherwise, if type is "error",
        if type_string == "error" {
            // Perform ! ReadableStreamDefaultControllerError(controller, value).
            self.controller.error(value.handle(), can_gc);

            // Disentangle port.
            global.disentangle_port(port, can_gc);
        }
    }

    /// <https://streams.spec.whatwg.org/#abstract-opdef-setupcrossrealmtransformwritable>
    /// Add a handler for port’s messageerror event with the following steps:
    pub(crate) fn handle_error(
        &self,
        cx: SafeJSContext,
        global: &GlobalScope,
        port: &MessagePort,
        _realm: InRealm,
        can_gc: CanGc,
    ) {
        // Let error be a new "DataCloneError" DOMException.
        let error = DOMException::new(global, DOMErrorName::DataCloneError, can_gc);
        rooted!(in(*cx) let mut rooted_error = UndefinedValue());
        error.safe_to_jsval(cx, rooted_error.handle_mut(), can_gc);

        // Perform ! CrossRealmTransformSendError(port, error).
        port.cross_realm_transform_send_error(rooted_error.handle(), can_gc);

        // Perform ! ReadableStreamDefaultControllerError(controller, error).
        self.controller.error(rooted_error.handle(), can_gc);

        // Disentangle port.
        global.disentangle_port(port, can_gc);
    }
}

#[expect(unsafe_code)]
/// Get the `done` property of an object that a read promise resolved to.
pub(crate) fn get_read_promise_done(
    cx: SafeJSContext,
    v: &SafeHandleValue,
    can_gc: CanGc,
) -> Result<bool, Error> {
    if !v.is_object() {
        return Err(Error::Type("Unknown format for done property.".to_string()));
    }
    unsafe {
        rooted!(in(*cx) let object = v.to_object());
        rooted!(in(*cx) let mut done = UndefinedValue());
        match get_dictionary_property(*cx, object.handle(), "done", done.handle_mut(), can_gc) {
            Ok(true) => match bool::safe_from_jsval(cx, done.handle(), (), can_gc) {
                Ok(ConversionResult::Success(val)) => Ok(val),
                Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.to_string())),
                _ => Err(Error::Type("Unknown format for done property.".to_string())),
            },
            Ok(false) => Err(Error::Type("Promise has no done property.".to_string())),
            Err(()) => Err(Error::JSFailed),
        }
    }
}

#[expect(unsafe_code)]
/// Get the `value` property of an object that a read promise resolved to.
pub(crate) fn get_read_promise_bytes(
    cx: SafeJSContext,
    v: &SafeHandleValue,
    can_gc: CanGc,
) -> Result<Vec<u8>, Error> {
    if !v.is_object() {
        return Err(Error::Type(
            "Unknown format for for bytes read.".to_string(),
        ));
    }
    unsafe {
        rooted!(in(*cx) let object = v.to_object());
        rooted!(in(*cx) let mut bytes = UndefinedValue());
        match get_dictionary_property(*cx, object.handle(), "value", bytes.handle_mut(), can_gc) {
            Ok(true) => {
                match Vec::<u8>::safe_from_jsval(
                    cx,
                    bytes.handle(),
                    ConversionBehavior::EnforceRange,
                    can_gc,
                ) {
                    Ok(ConversionResult::Success(val)) => Ok(val),
                    Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.to_string())),
                    _ => Err(Error::Type("Unknown format for bytes read.".to_string())),
                }
            },
            Ok(false) => Err(Error::Type("Promise has no value property.".to_string())),
            Err(()) => Err(Error::JSFailed),
        }
    }
}

/// Convert a raw stream `chunk` JS value to `Vec<u8>`.
/// This mirrors the conversion used inside `get_read_promise_bytes`,
/// but operates on the raw chunk (no `{ value, done }` wrapper).
pub(crate) fn bytes_from_chunk_jsval(
    cx: SafeJSContext,
    chunk: &RootedTraceableBox<Heap<JSVal>>,
    can_gc: CanGc,
) -> Result<Vec<u8>, Error> {
    match Vec::<u8>::safe_from_jsval(cx, chunk.handle(), ConversionBehavior::EnforceRange, can_gc) {
        Ok(ConversionResult::Success(vec)) => Ok(vec),
        Ok(ConversionResult::Failure(error)) => Err(Error::Type(error.to_string())),
        _ => Err(Error::Type("Unknown format for bytes read.".to_string())),
    }
}

/// <https://streams.spec.whatwg.org/#rs-transfer>
impl Transferable for ReadableStream {
    type Index = MessagePortIndex;
    type Data = MessagePortImpl;

    /// <https://streams.spec.whatwg.org/#ref-for-transfer-steps>
    fn transfer(&self) -> Fallible<(MessagePortId, MessagePortImpl)> {
        // Step 1. If ! IsReadableStreamLocked(value) is true, throw a
        // "DataCloneError" DOMException.
        if self.is_locked() {
            return Err(Error::DataClone(None));
        }

        let global = self.global();
        let realm = enter_realm(&*global);
        let comp = InRealm::Entered(&realm);
        let cx = GlobalScope::get_cx();
        let can_gc = CanGc::note();

        // Step 2. Let port1 be a new MessagePort in the current Realm.
        let port_1 = MessagePort::new(&global, can_gc);
        global.track_message_port(&port_1, None);

        // Step 3. Let port2 be a new MessagePort in the current Realm.
        let port_2 = MessagePort::new(&global, can_gc);
        global.track_message_port(&port_2, None);

        // Step 4. Entangle port1 and port2.
        global.entangle_ports(*port_1.message_port_id(), *port_2.message_port_id());

        // Step 5. Let writable be a new WritableStream in the current Realm.
        let writable = WritableStream::new_with_proto(&global, None, can_gc);

        // Step 6. Perform ! SetUpCrossRealmTransformWritable(writable, port1).
        writable.setup_cross_realm_transform_writable(cx, &port_1, can_gc);

        // Step 7. Let promise be ! ReadableStreamPipeTo(value, writable, false, false, false).
        let promise = self.pipe_to(
            cx, &global, &writable, false, false, false, None, comp, can_gc,
        );

        // Step 8. Set promise.[[PromiseIsHandled]] to true.
        promise.set_promise_is_handled();

        // Step 9. Set dataHolder.[[port]] to ! StructuredSerializeWithTransfer(port2, « port2 »).
        port_2.transfer()
    }

    /// <https://streams.spec.whatwg.org/#ref-for-transfer-receiving-steps>
    fn transfer_receive(
        owner: &GlobalScope,
        id: MessagePortId,
        port_impl: MessagePortImpl,
    ) -> Result<DomRoot<Self>, ()> {
        let cx = GlobalScope::get_cx();
        let can_gc = CanGc::note();

        // Their transfer-receiving steps, given dataHolder and value, are:
        // Note: dataHolder is used in `structuredclone.rs`, and value is created here.
        let value = ReadableStream::new_with_proto(owner, None, can_gc);

        // Step 1. Let deserializedRecord be !
        // StructuredDeserializeWithTransfer(dataHolder.[[port]], the current
        // Realm).
        // Done with the `Deserialize` derive of `MessagePortImpl`.

        // Step 2. Let port be deserializedRecord.[[Deserialized]].
        let transferred_port = MessagePort::transfer_receive(owner, id, port_impl)?;

        // Step 3. Perform ! SetUpCrossRealmTransformReadable(value, port).
        value.setup_cross_realm_transform_readable(cx, &transferred_port, can_gc);
        Ok(value)
    }

    /// Note: we are relying on the port transfer, so the data returned here are related to the port.
    fn serialized_storage<'a>(
        data: StructuredData<'a, '_>,
    ) -> &'a mut Option<FxHashMap<MessagePortId, Self::Data>> {
        match data {
            StructuredData::Reader(r) => &mut r.port_impls,
            StructuredData::Writer(w) => &mut w.ports,
        }
    }
}
