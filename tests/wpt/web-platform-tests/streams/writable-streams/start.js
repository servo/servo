'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
  self.importScripts('../resources/test-utils.js');
  self.importScripts('../resources/recording-streams.js');
}

promise_test(() => {
  let resolveStartPromise;
  const ws = recordingWritableStream({
    start() {
      return new Promise(resolve => {
        resolveStartPromise = resolve;
      });
    }
  });

  const writer = ws.getWriter();

  assert_equals(writer.desiredSize, 1, 'desiredSize should be 1');
  writer.write('a');
  assert_equals(writer.desiredSize, 0, 'desiredSize should be 0 after writer.write()');

  // Wait and verify that write isn't called.
  return flushAsyncEvents()
      .then(() => {
        assert_array_equals(ws.events, [], 'write should not be called until start promise resolves');
        resolveStartPromise();
        return writer.ready;
      })
      .then(() => assert_array_equals(ws.events, ['write', 'a'],
                                      'write should not be called until start promise resolves'));
}, 'underlying sink\'s write should not be called until start finishes');

promise_test(() => {
  let resolveStartPromise;
  const ws = recordingWritableStream({
    start() {
      return new Promise(resolve => {
        resolveStartPromise = resolve;
      });
    }
  });

  const writer = ws.getWriter();

  writer.close();
  assert_equals(writer.desiredSize, 1, 'desiredSize should be 1');

  // Wait and verify that write isn't called.
  return flushAsyncEvents().then(() => {
    assert_array_equals(ws.events, [], 'close should not be called until start promise resolves');
    resolveStartPromise();
    return writer.closed;
  });
}, 'underlying sink\'s close should not be called until start finishes');

test(() => {
  const passedError = new Error('horrible things');

  let writeCalled = false;
  let closeCalled = false;
  assert_throws(passedError, () => {
    // recordingWritableStream cannot be used here because the exception in the
    // constructor prevents assigning the object to a variable.
    new WritableStream({
      start() {
        throw passedError;
      },
      write() {
        writeCalled = true;
      },
      close() {
        closeCalled = true;
      }
    });
  }, 'constructor should throw passedError');
  assert_false(writeCalled, 'write should not be called');
  assert_false(closeCalled, 'close should not be called');
}, 'underlying sink\'s write or close should not be called if start throws');

promise_test(() => {
  const ws = recordingWritableStream({
    start() {
      return Promise.reject();
    }
  });

  // Wait and verify that write or close aren't called.
  return flushAsyncEvents()
      .then(() => assert_array_equals(ws.events, [], 'write and close should not be called'));
}, 'underlying sink\'s write or close should not be invoked if the promise returned by start is rejected');

promise_test(t => {
  const rejection = { name: 'this is checked' };
  const ws = new WritableStream({
    start() {
      return {
        then(onFulfilled, onRejected) { onRejected(rejection); }
      };
    }
  });
  return promise_rejects(t, rejection, ws.getWriter().closed, 'closed promise should be rejected');
}, 'returning a thenable from start() should work');

done();
