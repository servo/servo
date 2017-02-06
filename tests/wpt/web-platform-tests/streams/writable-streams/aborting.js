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

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return new Promise(() => { }); // forever-pending, so normally .ready would not fulfill.
    }
  });

  const writer = ws.getWriter();
  const writePromise = writer.write('a');

  const readyPromise = writer.ready;

  writer.abort(error1);

  assert_equals(writer.ready, readyPromise, 'the ready promise property should not change');

  return Promise.all([
    promise_rejects(t, new TypeError(), readyPromise, 'the ready promise should reject with a TypeError'),
    promise_rejects(t, new TypeError(), writePromise, 'the write() promise should reject with a TypeError')
  ]);
}, 'Aborting a WritableStream before it starts should cause the writer\'s unsettled ready promise to reject');

promise_test(t => {
  const ws = new WritableStream();

  const writer = ws.getWriter();
  writer.write('a');

  const readyPromise = writer.ready;

  return readyPromise.then(() => {
    writer.abort(error1);

    assert_not_equals(writer.ready, readyPromise, 'the ready promise property should change');
    return promise_rejects(t, new TypeError(), writer.ready, 'the ready promise should reject with a TypeError');
  });
}, 'Aborting a WritableStream should cause the writer\'s fulfilled ready promise to reset to a rejected one');

promise_test(t => {
  const ws = new WritableStream();
  const writer = ws.getWriter();

  writer.releaseLock();

  return promise_rejects(t, new TypeError(), writer.abort(), 'abort() should reject with a TypeError');
}, 'abort() on a released writer rejects');

promise_test(t => {
  const ws = recordingWritableStream();

  return delay(0)
    .then(() => {
      const writer = ws.getWriter();

      writer.abort();

      return Promise.all([
        promise_rejects(t, new TypeError(), writer.write(1), 'write(1) must reject with a TypeError'),
        promise_rejects(t, new TypeError(), writer.write(2), 'write(2) must reject with a TypeError')
      ]);
    })
    .then(() => {
      assert_array_equals(ws.events, ['abort', undefined]);
    });
}, 'Aborting a WritableStream immediately prevents future writes');

promise_test(t => {
  const ws = recordingWritableStream();
  const results = [];

  return delay(0)
    .then(() => {
      const writer = ws.getWriter();

      results.push(
        writer.write(1),
        promise_rejects(t, new TypeError(), writer.write(2), 'write(2) must reject with a TypeError'),
        promise_rejects(t, new TypeError(), writer.write(3), 'write(3) must reject with a TypeError')
      );

      const abortPromise = writer.abort();

      results.push(
        promise_rejects(t, new TypeError(), writer.write(4), 'write(4) must reject with a TypeError'),
        promise_rejects(t, new TypeError(), writer.write(5), 'write(5) must reject with a TypeError')
      );

      return abortPromise;
    }).then(() => {
      assert_array_equals(ws.events, ['write', 1, 'abort', undefined]);

      return Promise.all(results);
    });
}, 'Aborting a WritableStream prevents further writes after any that are in progress');

promise_test(() => {
  const ws = new WritableStream({
    abort() {
      return 'Hello';
    }
  });
  const writer = ws.getWriter();

  return writer.abort('a').then(value => {
    assert_equals(value, undefined, 'fulfillment value must be undefined');
  });
}, 'Fulfillment value of ws.abort() call must be undefined even if the underlying sink returns a non-undefined value');

promise_test(t => {
  const ws = new WritableStream({
    abort() {
      throw error1;
    }
  });
  const writer = ws.getWriter();

  return promise_rejects(t, error1, writer.abort(undefined),
    'rejection reason of abortPromise must be the error thrown by abort');
}, 'WritableStream if sink\'s abort throws, the promise returned by writer.abort() rejects');

promise_test(t => {
  const ws = new WritableStream({
    abort() {
      throw error1;
    }
  });

  return promise_rejects(t, error1, ws.abort(undefined),
    'rejection reason of abortPromise must be the error thrown by abort');
}, 'WritableStream if sink\'s abort throws, the promise returned by ws.abort() rejects');

promise_test(t => {
  let resolveWritePromise;
  const ws = new WritableStream({
    write() {
      return new Promise(resolve => {
        resolveWritePromise = resolve;
      });
    },
    abort() {
      throw error1;
    }
  });

  const writer = ws.getWriter();

  writer.write().catch(() => {});
  return flushAsyncEvents().then(() => {
    const abortPromise = writer.abort(undefined);

    resolveWritePromise();
    return promise_rejects(t, error1, abortPromise,
      'rejection reason of abortPromise must be the error thrown by abort');
  });
}, 'WritableStream if sink\'s abort throws, for an abort performed during a write, the promise returned by ' +
   'ws.abort() rejects');

test(() => {
  const ws = recordingWritableStream();
  const writer = ws.getWriter();

  writer.abort(error1);

  assert_array_equals(ws.events, ['abort', error1]);
}, 'Aborting a WritableStream passes through the given reason');

promise_test(t => {
  const ws = new WritableStream();
  const writer = ws.getWriter();

  writer.abort(error1);

  const events = [];
  writer.ready.catch(() => {
    events.push('ready');
  });
  writer.closed.catch(() => {
    events.push('closed');
  });

  return Promise.all([
    promise_rejects(t, new TypeError(), writer.write(), 'writing should reject with a TypeError'),
    promise_rejects(t, new TypeError(), writer.close(), 'closing should reject with a TypeError'),
    promise_rejects(t, new TypeError(), writer.abort(), 'aborting should reject with a TypeError'),
    promise_rejects(t, new TypeError(), writer.ready, 'ready should reject with a TypeError'),
    promise_rejects(t, new TypeError(), writer.closed, 'closed should reject with a TypeError')
  ]).then(() => {
    assert_array_equals(['ready', 'closed'], events, 'ready should reject before closed');
  });
}, 'Aborting a WritableStream puts it in an errored state, with a TypeError as the stored error');

promise_test(t => {
  const ws = new WritableStream();
  const writer = ws.getWriter();

  const writePromise = promise_rejects(t, new TypeError(), writer.write('a'),
    'writing should reject with a TypeError');

  writer.abort(error1);

  return writePromise;
}, 'Aborting a WritableStream causes any outstanding write() promises to be rejected with a TypeError');

promise_test(t => {
  const ws = new WritableStream();
  const writer = ws.getWriter();

  const closePromise = writer.close();
  writer.abort(error1);

  return Promise.all([
    promise_rejects(t, new TypeError(), writer.closed, 'closed should reject with a TypeError'),
    promise_rejects(t, new TypeError(), closePromise, 'close() should reject with a TypeError')
  ]);
}, 'Closing but then immediately aborting a WritableStream causes the stream to error');

promise_test(t => {
  let resolveClose;
  const ws = new WritableStream({
    close() {
      return new Promise(resolve => {
        resolveClose = resolve;
      });
    }
  });
  const writer = ws.getWriter();

  const closePromise = writer.close();

  return delay(0).then(() => {
    const abortPromise = writer.abort(error1);
    resolveClose();
    return Promise.all([
      promise_rejects(t, new TypeError(), writer.closed, 'closed should reject with a TypeError'),
      abortPromise,
      closePromise
    ]);
  });
}, 'Closing a WritableStream and aborting it while it closes causes the stream to error');

promise_test(() => {
  const ws = new WritableStream();
  const writer = ws.getWriter();

  writer.close();

  return delay(0).then(() => writer.abort());
}, 'Aborting a WritableStream after it is closed is a no-op');

promise_test(t => {
  // Testing that per https://github.com/whatwg/streams/issues/620#issuecomment-263483953 the fallback to close was
  // removed.

  // Cannot use recordingWritableStream since it always has an abort
  let closeCalled = false;
  const ws = new WritableStream({
    close() {
      closeCalled = true;
    }
  });

  const writer = ws.getWriter();

  writer.abort();

  return promise_rejects(t, new TypeError(), writer.closed, 'closed should reject with a TypeError').then(() => {
    assert_false(closeCalled, 'close must not have been called');
  });
}, 'WritableStream should NOT call underlying sink\'s close if no abort is supplied (historical)');

promise_test(() => {
  let thenCalled = false;
  const ws = new WritableStream({
    abort() {
      return {
        then(onFulfilled) {
          thenCalled = true;
          onFulfilled();
        }
      };
    }
  });
  const writer = ws.getWriter();
  return writer.abort().then(() => assert_true(thenCalled, 'then() should be called'));
}, 'returning a thenable from abort() should work');

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return flushAsyncEvents();
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const writePromise = writer.write('a');
    writer.abort(error1);
    let closedResolved = false;
    return Promise.all([
      writePromise.then(() => assert_false(closedResolved, '.closed should not resolve before write()')),
      promise_rejects(t, new TypeError(), writer.closed, '.closed should reject').then(() => {
        closedResolved = true;
      })
    ]);
  });
}, '.closed should not resolve before fulfilled write()');

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return Promise.reject(error1);
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const writePromise = writer.write('a');
    const abortPromise = writer.abort(error2);
    let closedResolved = false;
    return Promise.all([
      promise_rejects(t, error1, writePromise, 'write() should reject')
          .then(() => assert_false(closedResolved, '.closed should not resolve before write()')),
      promise_rejects(t, error1, writer.closed, '.closed should reject')
          .then(() => {
            closedResolved = true;
          }),
      promise_rejects(t, error1, abortPromise, 'abort() should reject')]);
  });
}, '.closed should not resolve before rejected write(); write() error should overwrite abort() error');

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return flushAsyncEvents();
    }
  }, new CountQueuingStrategy(4));
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const settlementOrder = [];
    return Promise.all([
      writer.write('1').then(() => settlementOrder.push(1)),
      promise_rejects(t, new TypeError(), writer.write('2'), 'first queued write should be rejected')
          .then(() => settlementOrder.push(2)),
      promise_rejects(t, new TypeError(), writer.write('3'), 'second queued write should be rejected')
          .then(() => settlementOrder.push(3)),
      writer.abort(error1)
    ]).then(() => assert_array_equals([1, 2, 3], settlementOrder, 'writes should be satisfied in order'));
  });
}, 'writes should be satisfied in order when aborting');

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return Promise.reject(error1);
    }
  }, new CountQueuingStrategy(4));
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const settlementOrder = [];
    return Promise.all([
      promise_rejects(t, error1, writer.write('1'), 'pending write should be rejected')
          .then(() => settlementOrder.push(1)),
      promise_rejects(t, error1, writer.write('2'), 'first queued write should be rejected')
          .then(() => settlementOrder.push(2)),
      promise_rejects(t, error1, writer.write('3'), 'second queued write should be rejected')
          .then(() => settlementOrder.push(3)),
      promise_rejects(t, error1, writer.abort(error1), 'abort should be rejected')
    ]).then(() => assert_array_equals([1, 2, 3], settlementOrder, 'writes should be satisfied in order'));
  });
}, 'writes should be satisfied in order after rejected write when aborting');

promise_test(t => {
  const ws = new WritableStream({
    write() {
      return Promise.reject(error1);
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    return Promise.all([
      promise_rejects(t, error1, writer.write('a'), 'writer.write() should reject with error from underlying write()'),
      promise_rejects(t, error1, writer.close(), 'writer.close() should reject with error from underlying write()'),
      promise_rejects(t, error1, writer.abort(), 'writer.abort() should reject with error from underlying write()')
    ]);
  });
}, 'close() should use error from underlying write() on abort');

promise_test(() => {
  let resolveWrite;
  let abortCalled = false;
  const ws = new WritableStream({
    write() {
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    },
    abort() {
      abortCalled = true;
    }
  });

  const writer = ws.getWriter();
  return writer.ready.then(() => {
    writer.write('a');
    const abortPromise = writer.abort();
    return flushAsyncEvents().then(() => {
      assert_false(abortCalled, 'abort should not be called while write is pending');
      resolveWrite();
      return abortPromise.then(() => assert_true(abortCalled, 'abort should be called'));
    });
  });
}, 'underlying abort() should not be called until underlying write() completes');

promise_test(() => {
  let resolveClose;
  let abortCalled = false;
  const ws = new WritableStream({
    close() {
      return new Promise(resolve => {
        resolveClose = resolve;
      });
    },
    abort() {
      abortCalled = true;
    }
  });

  const writer = ws.getWriter();
  return writer.ready.then(() => {
    writer.close();
    const abortPromise = writer.abort();
    return flushAsyncEvents().then(() => {
      assert_false(abortCalled, 'underlying abort should not be called while close is pending');
      resolveClose();
      return abortPromise.then(() => {
        assert_false(abortCalled, 'underlying abort should not be called after close completes');
      });
    });
  });
}, 'underlying abort() should not be called if underlying close() has started');

promise_test(t => {
  let rejectClose;
  let abortCalled = false;
  const ws = new WritableStream({
    close() {
      return new Promise((resolve, reject) => {
        rejectClose = reject;
      });
    },
    abort() {
      abortCalled = true;
    }
  });

  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const closePromise = writer.close();
    const abortPromise = writer.abort();
    return flushAsyncEvents().then(() => {
      assert_false(abortCalled, 'underlying abort should not be called while close is pending');
      rejectClose(error1);
      return promise_rejects(t, error1, abortPromise, 'abort should reject with the same reason').then(() => {
        return promise_rejects(t, error1, closePromise, 'close should reject with the same reason');
      }).then(() => {
        assert_false(abortCalled, 'underlying abort should not be called after close completes');
      });
    });
  });
}, 'if underlying close() has started and then rejects, the abort() and close() promises should reject with the ' +
   'underlying close rejection reason');

promise_test(t => {
  let resolveWrite;
  let abortCalled = false;
  const ws = new WritableStream({
    write() {
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    },
    abort() {
      abortCalled = true;
    }
  });

  const writer = ws.getWriter();
  return writer.ready.then(() => {
    writer.write('a');
    const closePromise = writer.close();
    const abortPromise = writer.abort();
    return flushAsyncEvents().then(() => {
      assert_false(abortCalled, 'abort should not be called while write is pending');
      resolveWrite();
      return abortPromise.then(() => {
        assert_true(abortCalled, 'abort should be called after write completes');
        return promise_rejects(t, new TypeError(), closePromise, 'promise returned by close() should be rejected');
      });
    });
  });
}, 'underlying abort() should be called while closing if underlying close() has not started yet');

promise_test(() => {
  const ws = new WritableStream();
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const closePromise = writer.close();
    const abortPromise = writer.abort();
    let closeResolved = false;
    Promise.all([
      closePromise.then(() => { closeResolved = true; }),
      abortPromise.then(() => { assert_true(closeResolved, 'close() promise should resolve before abort() promise'); })
    ]);
  });
}, 'writer close() promise should resolve before abort() promise');

promise_test(t => {
  const ws = new WritableStream({
    write(chunk, controller) {
      controller.error(error1);
      return new Promise(() => {});
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    writer.write('a');
    return promise_rejects(t, error1, writer.ready, 'writer.ready should reject');
  });
}, 'writer.ready should reject on controller error without waiting for underlying write');

promise_test(t => {
  let rejectWrite;
  const ws = new WritableStream({
    write() {
      return new Promise((resolve, reject) => {
        rejectWrite = reject;
      });
    }
  });

  let writePromise;
  let abortPromise;

  const events = [];

  const writer = ws.getWriter();

  writer.closed.catch(() => {
    events.push('closed');
  });

  // Wait for ws to start
  return flushAsyncEvents().then(() => {
    writePromise = writer.write('a');
    writePromise.catch(() => {
      events.push('writePromise');
    });

    abortPromise = writer.abort(error1);
    abortPromise.catch(() => {
      events.push('abortPromise');
    });

    const writePromise2 = writer.write('a');

    return Promise.all([
      promise_rejects(t, new TypeError(), writePromise2, 'writePromise2 must reject with an error indicating abort'),
      promise_rejects(t, new TypeError(), writer.ready, 'writer.ready must reject with an error indicating abort'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, [], 'writePromise, abortPromise and writer.closed must not be rejected yet');

    rejectWrite(error2);

    return Promise.all([
      promise_rejects(t, error2, writePromise,
                      'writePromise must reject with the error returned from the sink\'s write method'),
      promise_rejects(t, error2, abortPromise,
                      'abortPromise must reject with the error returned from the sink\'s write method'),
      promise_rejects(t, error2, writer.closed,
                      'writer.closed must reject with the error returned from the sink\'s write method'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, ['writePromise', 'abortPromise', 'closed'],
                        'writePromise, abortPromise and writer.closed must reject');

    const writePromise3 = writer.write('a');

    return Promise.all([
      promise_rejects(t, new TypeError(), writePromise3,
                      'writePromise3 must reject with an error indicating the stream has already been errored'),
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be still rejected with the error indicating abort')
    ]);
  }).then(() => {
    writer.releaseLock();

    return Promise.all([
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be rejected with an error indicating release'),
      promise_rejects(t, new TypeError(), writer.closed,
                      'writer.closed must be rejected with an error indicating release')
    ]);
  });
}, 'writer.abort() while there is a pending write, and then finish the write with rejection');

promise_test(t => {
  let resolveWrite;
  let controller;
  const ws = new WritableStream({
    write(chunk, c) {
      controller = c;
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    }
  });

  let writePromise;
  let abortPromise;

  const events = [];

  const writer = ws.getWriter();

  writer.closed.catch(() => {
    events.push('closed');
  });

  // Wait for ws to start
  return flushAsyncEvents().then(() => {
    writePromise = writer.write('a');
    writePromise.then(() => {
      events.push('writePromise');
    });

    abortPromise = writer.abort(error1);
    abortPromise.catch(() => {
      events.push('abortPromise');
    });

    const writePromise2 = writer.write('a');

    return Promise.all([
      promise_rejects(t, new TypeError(), writePromise2, 'writePromise2 must reject with an error indicating abort'),
      promise_rejects(t, new TypeError(), writer.ready, 'writer.ready must reject with an error indicating abort'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, [], 'writePromise, abortPromise and writer.closed must not be fulfilled/rejected yet');

    controller.error(error2);

    const writePromise3 = writer.write('a');

    return Promise.all([
      promise_rejects(t, new TypeError(), writePromise3,
                      'writePromise3 must reject with an error indicating the stream has already been errored'),
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be still rejected with the error indicating abort'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(
        events, [],
        'writePromise, abortPromise and writer.closed must not be fulfilled/rejected yet even after '
            + 'controller.error() call');

    resolveWrite();

    return Promise.all([
      writePromise,
      promise_rejects(t, error2, abortPromise,
                      'abortPromise must reject with the error passed to the controller\'s error method'),
      promise_rejects(t, error2, writer.closed,
                      'writer.closed must reject with the error passed to the controller\'s error method'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, ['writePromise', 'abortPromise', 'closed'],
                        'writePromise, abortPromise and writer.closed must reject');

    const writePromise4 = writer.write('a');

    return Promise.all([
      writePromise,
      promise_rejects(t, new TypeError(), writePromise4,
                      'writePromise4 must reject with an error indicating that the stream has already been errored'),
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be still rejected with the error indicating abort')
    ]);
  }).then(() => {
    writer.releaseLock();

    return Promise.all([
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be rejected with an error indicating release'),
      promise_rejects(t, new TypeError(), writer.closed,
                      'writer.closed must be rejected with an error indicating release')
    ]);
  });
}, 'writer.abort(), controller.error() while there is a pending write, and then finish the write');

promise_test(t => {
  let resolveWrite;
  let controller;
  const ws = new WritableStream({
    write(chunk, c) {
      controller = c;
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    }
  });

  let writePromise;
  let abortPromise;

  const events = [];

  const writer = ws.getWriter();

  writer.closed.catch(() => {
    events.push('closed');
  });

  // Wait for ws to start
  return flushAsyncEvents().then(() => {
    writePromise = writer.write('a');
    writePromise.then(() => {
      events.push('writePromise');
    });

    controller.error(error2);

    const writePromise2 = writer.write('a');

    return Promise.all([
      promise_rejects(t, new TypeError(), writePromise2,
                      'writePromise2 must reject with an error indicating the stream has already been errored'),
      promise_rejects(t, error2, writer.ready,
                      'writer.ready must reject with the error passed to the controller\'s error method'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, [], 'writePromise and writer.closed must not be fulfilled/rejected yet');

    abortPromise = writer.abort(error1);
    abortPromise.catch(() => {
      events.push('abortPromise');
    });

    const writePromise3 = writer.write('a');

    return Promise.all([
      promise_rejects(t, error2, abortPromise,
                      'abortPromise must reject with the error passed to the controller\'s error method'),
      promise_rejects(t, new TypeError(), writePromise3,
                      'writePromise3 must reject with an error indicating the stream has already been errored'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(
        events, ['abortPromise'],
        'writePromise and writer.closed must not be fulfilled/rejected yet even after writer.abort()');

    resolveWrite();

    return Promise.all([
      promise_rejects(t, error2, writer.closed,
                      'writer.closed must reject with the error passed to the controller\'s error method'),
      flushAsyncEvents()
    ]);
  }).then(() => {
    assert_array_equals(events, ['abortPromise', 'writePromise', 'closed'],
                        'writePromise, abortPromise and writer.closed must fulfill/reject');

    const writePromise4 = writer.write('a');

    return Promise.all([
      writePromise,
      promise_rejects(t, new TypeError(), writePromise4,
                      'writePromise4 must reject with an error indicating that the stream has already been errored'),
      promise_rejects(t, error2, writer.ready,
                      'writer.ready must be still rejected with the error passed to the controller\'s error method')
    ]);
  }).then(() => {
    writer.releaseLock();

    return Promise.all([
      promise_rejects(t, new TypeError(), writer.ready,
                      'writer.ready must be rejected with an error indicating release'),
      promise_rejects(t, new TypeError(), writer.closed,
                      'writer.closed must be rejected with an error indicating release')
    ]);
  });
}, 'controller.error(), writer.abort() while there is a pending write, and then finish the write');

promise_test(t => {
  let resolveWrite;
  const ws = new WritableStream({
    write() {
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const writePromise = writer.write('a');
    const closed = writer.closed;
    const abortPromise = writer.abort();
    writer.releaseLock();
    resolveWrite();
    return Promise.all([
      writePromise,
      abortPromise,
      promise_rejects(t, new TypeError(), closed, 'closed should reject')]);
  });
}, 'releaseLock() while aborting should reject the original closed promise');

promise_test(t => {
  let resolveWrite;
  let resolveAbort;
  let resolveAbortStarted;
  const abortStarted = new Promise(resolve => {
    resolveAbortStarted = resolve;
  });
  const ws = new WritableStream({
    write() {
      return new Promise(resolve => {
        resolveWrite = resolve;
      });
    },
    abort() {
      resolveAbortStarted();
      return new Promise(resolve => {
        resolveAbort = resolve;
      });
    }
  });
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    const writePromise = writer.write('a');
    const closed = writer.closed;
    const abortPromise = writer.abort();
    resolveWrite();
    return abortStarted.then(() => {
      writer.releaseLock();
      assert_not_equals(writer.closed, closed, 'closed promise should have changed');
      resolveAbort();
      return Promise.all([
        writePromise,
        abortPromise,
        promise_rejects(t, new TypeError(), closed, 'original closed should reject'),
        promise_rejects(t, new TypeError(), writer.closed, 'new closed should reject')]);
    });
  });
}, 'releaseLock() during delayed async abort() should create a new rejected closed promise');

done();
