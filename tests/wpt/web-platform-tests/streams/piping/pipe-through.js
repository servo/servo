'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
  self.importScripts('../resources/rs-utils.js');
  self.importScripts('../resources/test-utils.js');
}

function duckTypedPassThroughTransform() {
  let enqueueInReadable;
  let closeReadable;

  return {
    writable: new WritableStream({
      write(chunk) {
        enqueueInReadable(chunk);
      },

      close() {
        closeReadable();
      }
    }),

    readable: new ReadableStream({
      start(c) {
        enqueueInReadable = c.enqueue.bind(c);
        closeReadable = c.close.bind(c);
      }
    })
  };
}

promise_test(() => {
  const readableEnd = sequentialReadableStream(5).pipeThrough(duckTypedPassThroughTransform());

  return readableStreamToArray(readableEnd).then(chunks =>
    assert_array_equals(chunks, [1, 2, 3, 4, 5]), 'chunks should match');
}, 'Piping through a duck-typed pass-through transform stream should work');

promise_test(t => {
  const transform = {
    writable: new WritableStream({
      start(c) {
        c.error(new Error('this rejection should not be reported as unhandled'));
      }
    }),
    readable: new ReadableStream()
  };

  sequentialReadableStream(5).pipeThrough(transform);

  // The test harness should complain about unhandled rejections by then.
  return flushAsyncEvents();

}, 'Piping through a transform errored on the writable end does not cause an unhandled promise rejection');

test(() => {
  let calledWithArgs;
  const dummy = {
    pipeTo(...args) {
      calledWithArgs = args;

      // Does not return anything, testing the spec's guard against trying to mark [[PromiseIsHandled]] on undefined.
    }
  };

  const fakeWritable = { fake: 'writable' };
  const fakeReadable = { fake: 'readable' };
  const arg2 = { arg: 'arg2' };
  const arg3 = { arg: 'arg3' };
  ReadableStream.prototype.pipeThrough.call(dummy, { writable: fakeWritable, readable: fakeReadable }, arg2, arg3);

  assert_array_equals(calledWithArgs, [fakeWritable, arg2],
    'The this value\'s pipeTo method should be called with the appropriate arguments');

}, 'pipeThrough generically calls pipeTo with the appropriate args');

test(() => {
  const dummy = {
    pipeTo(args) {
      return { not: 'a promise' };
    }
  };

  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // Test passes if this doesn't throw or crash.

}, 'pipeThrough can handle calling a pipeTo that returns a non-promise object');

test(() => {
  const dummy = {
    pipeTo(args) {
      return {
        then() {},
        this: 'is not a real promise'
      };
    }
  };

  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // Test passes if this doesn't throw or crash.

}, 'pipeThrough can handle calling a pipeTo that returns a non-promise thenable object');

done();
