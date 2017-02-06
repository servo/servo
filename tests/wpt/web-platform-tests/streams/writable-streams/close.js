'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
  self.importScripts('../resources/test-utils.js');
  self.importScripts('../resources/recording-streams.js');
}

const error1 = new Error('error1');
error1.name = 'error1';

const error2 = new Error('error2');
error2.name = 'error2';

promise_test(() => {
  const ws = new WritableStream({
    close() {
      return 'Hello';
    }
  });

  const writer = ws.getWriter();

  const closePromise = writer.close();
  return closePromise.then(value => assert_equals(value, undefined, 'fulfillment value must be undefined'));
}, 'fulfillment value of ws.close() call must be undefined even if the underlying sink returns a non-undefined ' +
    'value');

promise_test(t => {
  const passedError = new Error('error me');
  let controller;
  const ws = new WritableStream({
    close(c) {
      controller = c;
      return delay(50);
    }
  });

  const writer = ws.getWriter();

  return Promise.all([
    writer.close(),
    delay(10).then(() => controller.error(passedError)),
    promise_rejects(t, passedError, writer.closed,
                    'closed promise should be rejected with the passed error'),
    delay(70).then(() => promise_rejects(t, passedError, writer.closed, 'closed should stay rejected'))
  ]);
}, 'when sink calls error asynchronously while closing, the stream should become errored');

promise_test(t => {
  const passedError = new Error('error me');
  const ws = new WritableStream({
    close(controller) {
      controller.error(passedError);
    }
  });

  const writer = ws.getWriter();

  return writer.close().then(() => promise_rejects(t, passedError, writer.closed, 'closed should stay rejected'));
}, 'when sink calls error synchronously while closing, the stream should become errored');

promise_test(t => {
  const ws = new WritableStream({
    write(chunk, controller) {
      controller.error(error1);
      return new Promise(() => {});
    }
  });

  const writer = ws.getWriter();
  writer.write('a');

  return delay(0).then(() => {
    writer.releaseLock();
  });
}, 'releaseLock on a stream with a pending write in which the stream has been errored');

promise_test(t => {
  const ws = new WritableStream({
    close(controller) {
      controller.error(error1);
      return new Promise(() => {});
    }
  });

  const writer = ws.getWriter();
  writer.close();

  return delay(0).then(() => {
    writer.releaseLock();
  });
}, 'releaseLock on a stream with a pending close in which the stream has been errored');

promise_test(() => {
  const ws = recordingWritableStream();

  const writer = ws.getWriter();

  return writer.ready.then(() => {
    assert_equals(writer.desiredSize, 1, 'desiredSize should be 1');

    writer.close();
    assert_equals(writer.desiredSize, 1, 'desiredSize should be still 1');

    return writer.ready.then(v => {
      assert_equals(v, undefined, 'ready promise should be fulfilled with undefined');
      assert_array_equals(ws.events, ['close'], 'write and abort should not be called');
    });
  });
}, 'when close is called on a WritableStream in writable state, ready should return a fulfilled promise');

promise_test(() => {
  const ws = recordingWritableStream({
    write() {
      return new Promise(() => {});
    }
  });

  const writer = ws.getWriter();

  return writer.ready.then(() => {
    writer.write('a');

    assert_equals(writer.desiredSize, 0, 'desiredSize should be 0');

    let calledClose = false;
    return Promise.all([
      writer.ready.then(v => {
        assert_equals(v, undefined, 'ready promise should be fulfilled with undefined');
        assert_true(calledClose, 'ready should not be fulfilled before writer.close() is called');
        assert_array_equals(ws.events, ['write', 'a'], 'sink abort() should not be called');
      }),
      flushAsyncEvents().then(() => {
        writer.close();
        calledClose = true;
      })
    ]);
  });
}, 'when close is called on a WritableStream in waiting state, ready promise should be fulfilled');

promise_test(() => {
  let asyncCloseFinished = false;
  const ws = recordingWritableStream({
    close() {
      return flushAsyncEvents().then(() => {
        asyncCloseFinished = true;
      });
    }
  });

  const writer = ws.getWriter();
  return writer.ready.then(() => {
    writer.write('a');

    writer.close();

    return writer.ready.then(v => {
      assert_false(asyncCloseFinished, 'ready promise should be fulfilled before async close completes');
      assert_equals(v, undefined, 'ready promise should be fulfilled with undefined');
      assert_array_equals(ws.events, ['write', 'a', 'close'], 'sink abort() should not be called');
    });
  });
}, 'when close is called on a WritableStream in waiting state, ready should be fulfilled immediately even if close ' +
    'takes a long time');

promise_test(t => {
  const rejection = { name: 'letter' };
  const ws = new WritableStream({
    close() {
      return {
        then(onFulfilled, onRejected) { onRejected(rejection); }
      };
    }
  });
  return promise_rejects(t, rejection, ws.getWriter().close(), 'close() should return a rejection');
}, 'returning a thenable from close() should work');

promise_test(t => {
  const ws = new WritableStream();
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const closePromise = writer.close();
    const closedPromise = writer.closed;
    writer.releaseLock();
    return Promise.all([
      closePromise,
      promise_rejects(t, new TypeError(), closedPromise, '.closed promise should be rejected')
    ]);
  });
}, 'releaseLock() should not change the result of sync close()');

promise_test(t => {
  const ws = new WritableStream({
    close() {
      return flushAsyncEvents();
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const closePromise = writer.close();
    const closedPromise = writer.closed;
    writer.releaseLock();
    return Promise.all([
      closePromise,
      promise_rejects(t, new TypeError(), closedPromise, '.closed promise should be rejected')
    ]);
  });
}, 'releaseLock() should not change the result of async close()');

promise_test(() => {
  let resolveClose;
  const ws = new WritableStream({
    close() {
      const promise = new Promise(resolve => {
        resolveClose = resolve;
      });
      return promise;
    }
  });
  const writer = ws.getWriter();
  const closePromise = writer.close();
  writer.releaseLock();
  return delay(0).then(() => {
    resolveClose();
    return closePromise.then(() => {
      assert_equals(ws.getWriter().desiredSize, 0, 'desiredSize should be 0');
    });
  });
}, 'close() should set state to CLOSED even if writer has detached');

promise_test(() => {
  let resolveClose;
  const ws = new WritableStream({
    close() {
      const promise = new Promise(resolve => {
        resolveClose = resolve;
      });
      return promise;
    }
  });
  const writer = ws.getWriter();
  writer.close();
  writer.releaseLock();
  return delay(0).then(() => {
    const abortingWriter = ws.getWriter();
    const abortPromise = abortingWriter.abort();
    abortingWriter.releaseLock();
    resolveClose();
    return abortPromise;
  });
}, 'the promise returned by async abort during close should resolve');

// Though the order in which the promises are fulfilled or rejected is arbitrary, we're checking it for
// interoperability. We can change the order as long as we file bugs on all implementers to update to the latest tests
// to keep them interoperable.

promise_test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();

  const closePromise = writer.close();

  const events = [];
  return Promise.all([
    closePromise.then(() => {
      events.push('closePromise');
    }),
    writer.closed.then(() => {
      events.push('closed');
    })
  ]).then(() => {
    assert_array_equals(events, ['closePromise', 'closed'],
                        'promises must fulfill/reject in the expected order');
  });
}, 'promises must fulfill/reject in the expected order on closure');

promise_test(t => {
  const ws = new WritableStream({});

  // Wait until the WritableStream starts so that the close() call gets processed. Otherwise, abort() will be
  // processed without waiting for completion of the close().
  return delay(0).then(() => {
    const writer = ws.getWriter();

    const closePromise = writer.close();
    const abortPromise = writer.abort(error1);

    const events = [];
    return Promise.all([
      closePromise.then(() => {
        events.push('closePromise');
      }),
      abortPromise.then(() => {
        events.push('abortPromise');
      }),
      promise_rejects(t, new TypeError(), writer.closed, 'writer.closed must reject with an error indicating abort')
      .then(() => {
        events.push('closed');
      })
    ]).then(() => {
      assert_array_equals(events, ['closePromise', 'abortPromise', 'closed'],
                          'promises must fulfill/reject in the expected order');
    });
  });
}, 'promises must fulfill/reject in the expected order on aborted closure');

promise_test(t => {
  const ws = new WritableStream({
    close() {
      return Promise.reject(error1);
    }
  });

  // Wait until the WritableStream starts so that the close() call gets processed.
  return delay(0).then(() => {
    const writer = ws.getWriter();

    const closePromise = writer.close();
    const abortPromise = writer.abort(error2);

    const events = [];
    return Promise.all([
      promise_rejects(t, error1, closePromise,
                      'closePromise must reject with the error returned from the sink\'s close method')
      .then(() => {
        events.push('closePromise');
      }),
      promise_rejects(t, error1, abortPromise,
                      'abortPromise must reject with the error returned from the sink\'s close method')
      .then(() => {
        events.push('abortPromise');
      }),
      promise_rejects(t, error1, writer.closed,
                      'writer.closed must reject with the error returned from the sink\'s close method')
      .then(() => {
        events.push('closed');
      })
    ]).then(() => {
      assert_array_equals(events, ['closePromise', 'abortPromise', 'closed'],
                          'promises must fulfill/reject in the expected order');
    });
  });
}, 'promises must fulfill/reject in the expected order on aborted and errored closure');

done();
