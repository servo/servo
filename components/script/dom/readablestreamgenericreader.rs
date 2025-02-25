/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use js::jsval::UndefinedValue;
use js::rust::HandleValue as SafeHandleValue;

use super::readablestream::ReaderType;
use super::types::ReadableStream;
use crate::dom::bindings::error::{Error, ErrorToJsval};
use crate::dom::bindings::import::module::Fallible;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::readablestreambyobreader::ReadableStreamBYOBReader;
use crate::dom::readablestreamdefaultreader::ReadableStreamDefaultReader;
use crate::script_runtime::CanGc;

/// <https://streams.spec.whatwg.org/#readablestreamgenericreader>
pub(crate) trait ReadableStreamGenericReader {
    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-initialize>
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn generic_initialize(&self, global: &GlobalScope, stream: &ReadableStream, can_gc: CanGc) {
        // Set reader.[[stream]] to stream.
        self.set_stream(Some(stream));

        // Set stream.[[reader]] to reader.
        let reader_type = if let Some(default_reader) = self.as_default_reader() {
            ReaderType::Default(MutNullableDom::new(Some(default_reader)))
        } else if let Some(byob_reader) = self.as_byob_reader() {
            ReaderType::BYOB(MutNullableDom::new(Some(byob_reader)))
        } else {
            unreachable!("Reader must be either Default or BYOB.");
        };
        stream.set_reader(Some(reader_type));

        if stream.is_readable() {
            // If stream.[[state]] is "readable
            // Set reader.[[closedPromise]] to a new promise.
            self.set_closed_promise(Promise::new(global, can_gc));
        } else if stream.is_closed() {
            // Otherwise, if stream.[[state]] is "closed",
            // Set reader.[[closedPromise]] to a promise resolved with undefined.
            let cx = GlobalScope::get_cx();
            self.set_closed_promise(Promise::new_resolved(global, cx, (), can_gc));
        } else {
            // Assert: stream.[[state]] is "errored"
            assert!(stream.is_errored());

            // Set reader.[[closedPromise]] to a promise rejected with stream.[[storedError]].
            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut error = UndefinedValue());
            stream.get_stored_error(error.handle_mut());
            self.set_closed_promise(Promise::new_rejected(global, cx, error.handle(), can_gc));

            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true
            self.get_closed_promise().set_promise_is_handled();
        }
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-cancel>
    fn generic_cancel(&self, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        // Let stream be reader.[[stream]].
        let stream = self.get_stream();

        // Assert: stream is not undefined.
        let stream =
            stream.expect("Reader should have a stream when generic cancel is called into.");

        // Return ! ReadableStreamCancel(stream, reason).
        stream.cancel(reason, can_gc)
    }

    /// <https://streams.spec.whatwg.org/#readable-stream-reader-generic-release>
    #[allow(unsafe_code)]
    fn generic_release(&self, can_gc: CanGc) -> Fallible<()> {
        // Let stream be reader.[[stream]].

        // Assert: stream is not undefined.
        assert!(self.get_stream().is_some());

        if let Some(stream) = self.get_stream() {
            // Assert: stream.[[reader]] is reader.
            assert!(stream.has_default_reader());

            if stream.is_readable() {
                // If stream.[[state]] is "readable", reject reader.[[closedPromise]] with a TypeError exception.
                self.get_closed_promise().reject_error(
                    Error::Type("stream state is not readable".to_owned()),
                    can_gc,
                );
            } else {
                // Otherwise, set reader.[[closedPromise]] to a promise rejected with a TypeError exception.
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut error = UndefinedValue());
                Error::Type("Cannot release lock due to stream state.".to_owned()).to_jsval(
                    cx,
                    &stream.global(),
                    error.handle_mut(),
                );

                self.set_closed_promise(Promise::new_rejected(
                    &stream.global(),
                    cx,
                    error.handle(),
                    can_gc,
                ));
            }
            // Set reader.[[closedPromise]].[[PromiseIsHandled]] to true.
            self.get_closed_promise().set_promise_is_handled();

            // Perform ! stream.[[controller]].[[ReleaseSteps]]().
            stream.perform_release_steps()?;

            // Set stream.[[reader]] to undefined.
            stream.set_reader(None);
            // Set reader.[[stream]] to undefined.
            self.set_stream(None);
        }
        Ok(())
    }

    /// <https://streams.spec.whatwg.org/#generic-reader-closed>
    fn closed(&self) -> Rc<Promise> {
        self.get_closed_promise()
    }

    // <https://streams.spec.whatwg.org/#generic-reader-cancel>
    fn cancel(&self, global: &GlobalScope, reason: SafeHandleValue, can_gc: CanGc) -> Rc<Promise> {
        if self.get_stream().is_none() {
            // If this.[[stream]] is undefined,
            // return a promise rejected with a TypeError exception.
            let promise = Promise::new(global, can_gc);
            promise.reject_error(Error::Type("stream is undefined".to_owned()), can_gc);
            promise
        } else {
            // Return ! ReadableStreamReaderGenericCancel(this, reason).
            self.generic_cancel(reason, can_gc)
        }
    }

    fn set_stream(&self, stream: Option<&ReadableStream>);

    fn get_stream(&self) -> Option<DomRoot<ReadableStream>>;

    fn set_closed_promise(&self, promise: Rc<Promise>);

    fn get_closed_promise(&self) -> Rc<Promise>;

    fn as_default_reader(&self) -> Option<&ReadableStreamDefaultReader> {
        None
    }

    fn as_byob_reader(&self) -> Option<&ReadableStreamBYOBReader> {
        None
    }
}
