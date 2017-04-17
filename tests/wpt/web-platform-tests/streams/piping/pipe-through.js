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

promise_test(() => {
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
    pipeTo() {
      return { not: 'a promise' };
    }
  };

  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // Test passes if this doesn't throw or crash.

}, 'pipeThrough can handle calling a pipeTo that returns a non-promise object');

test(() => {
  const dummy = {
    pipeTo() {
      return {
        then() {},
        this: 'is not a real promise'
      };
    }
  };

  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // Test passes if this doesn't throw or crash.

}, 'pipeThrough can handle calling a pipeTo that returns a non-promise thenable object');

promise_test(() => {
  const dummy = {
    pipeTo() {
      return Promise.reject(new Error('this rejection should not be reported as unhandled'));
    }
  };

  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // The test harness should complain about unhandled rejections by then.
  return flushAsyncEvents();

}, 'pipeThrough should mark a real promise from a fake readable as handled');

test(() => {
  let thenCalled = false
  let catchCalled = false;
  const dummy = {
    pipeTo() {
      const fakePromise = Object.create(Promise.prototype);
      fakePromise.then = () => {
        thenCalled = true;
      };
      fakePromise.catch = () => {
        catchCalled = true;
      };
      assert_true(fakePromise instanceof Promise, 'fakePromise fools instanceof');
      return fakePromise;
    }
  };

  // An incorrect implementation which uses an internal method to mark the promise as handled will throw or crash here.
  ReadableStream.prototype.pipeThrough.call(dummy, { });

  // An incorrect implementation that tries to mark the promise as handled by calling .then() or .catch() on the object
  // will fail these tests.
  assert_false(thenCalled, 'then should not be called');
  assert_false(catchCalled, 'catch should not be called');
}, 'pipeThrough should not be fooled by an object whose instanceof Promise returns true');

done();
