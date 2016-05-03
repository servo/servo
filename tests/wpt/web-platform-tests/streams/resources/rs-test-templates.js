'use strict';

// These tests can be run against any readable stream produced by the web platform that meets the given descriptions.
// For readable stream tests, the factory should return the stream. For reader tests, the factory should return a
// { stream, reader } object. (You can use this to vary the time at which you acquire a reader.)

self.templatedRSEmpty = (label, factory) => {
  test(() => {}, 'Running templatedRSEmpty with ' + label);

  test(() => {

    const rs = factory();

    assert_equals(typeof rs.locked, 'boolean', 'has a boolean locked getter');
    assert_equals(typeof rs.cancel, 'function', 'has a cancel method');
    assert_equals(typeof rs.getReader, 'function', 'has a getReader method');
    assert_equals(typeof rs.pipeThrough, 'function', 'has a pipeThrough method');
    assert_equals(typeof rs.pipeTo, 'function', 'has a pipeTo method');
    assert_equals(typeof rs.tee, 'function', 'has a tee method');

  }, 'instances have the correct methods and properties');

  test(() => {
    const rs = factory();

    assert_throws(new RangeError(), () => rs.getReader({ mode: '' }), 'empty string mode should throw');
    assert_throws(new RangeError(), () => rs.getReader({ mode: null }), 'null mode should throw');
    assert_throws(new RangeError(), () => rs.getReader({ mode: 'asdf' }), 'asdf mode should throw');
    assert_throws(new TypeError(), () => rs.getReader(null), 'null should throw');

  }, 'calling getReader with invalid arguments should throw appropriate errors');
};

self.templatedRSClosed = (label, factory) => {
  test(() => {}, 'Running templatedRSClosed with ' + label);

  promise_test(() => {

    const rs = factory();
    const cancelPromise1 = rs.cancel();
    const cancelPromise2 = rs.cancel();

    assert_not_equals(cancelPromise1, cancelPromise2, 'cancel() calls should return distinct promises');

    return Promise.all([
      cancelPromise1.then(v => assert_equals(v, undefined, 'first cancel() call should fulfill with undefined')),
      cancelPromise2.then(v => assert_equals(v, undefined, 'second cancel() call should fulfill with undefined'))
    ]);

  }, 'cancel() should return a distinct fulfilled promise each time');

  test(() => {

    const rs = factory();
    assert_false(rs.locked, 'locked getter should return false');

  }, 'locked should be false');

  test(() => {

    const rs = factory();
    rs.getReader(); // getReader() should not throw.

  }, 'getReader() should be OK');

  test(() => {

    const rs = factory();

    const reader = rs.getReader();
    reader.releaseLock();

    const reader2 = rs.getReader(); // Getting a second reader should not throw.
    reader2.releaseLock();

    rs.getReader(); // Getting a third reader should not throw.

  }, 'should be able to acquire multiple readers if they are released in succession');

  test(() => {

    const rs = factory();

    rs.getReader();

    assert_throws(new TypeError(), () => rs.getReader(), 'getting a second reader should throw');
    assert_throws(new TypeError(), () => rs.getReader(), 'getting a third reader should throw');

  }, 'should not be able to acquire a second reader if we don\'t release the first one');
};

self.templatedRSErrored = (label, factory, error) => {
  test(() => {}, 'Running templatedRSErrored with ' + label);

  promise_test(t => {

    const rs = factory();
    const reader = rs.getReader();

    return Promise.all([
      promise_rejects(t, error, reader.closed),
      promise_rejects(t, error, reader.read())
    ]);

  }, 'getReader() should return a reader that acts errored');

  promise_test(t => {

    const rs = factory();
    const reader = rs.getReader();

    return Promise.all([
      promise_rejects(t, error, reader.read()),
      promise_rejects(t, error, reader.read()),
      promise_rejects(t, error, reader.closed)
    ]);

  }, 'read() twice should give the error each time');

  test(() => {
    const rs = factory();

    assert_false(rs.locked, 'locked getter should return false');
  }, 'locked should be false');
};

self.templatedRSErroredSyncOnly = (label, factory, error) => {
  test(() => {}, 'Running templatedRSErroredSyncOnly with ' + label);

  promise_test(t => {

    const rs = factory();
    rs.getReader().releaseLock();
    const reader = rs.getReader(); // Calling getReader() twice does not throw (the stream is not locked).

    return promise_rejects(t, error, reader.closed);

  }, 'should be able to obtain a second reader, with the correct closed promise');

  test(() => {

    const rs = factory();
    rs.getReader();

    assert_throws(new TypeError(), () => rs.getReader(), 'getting a second reader should throw a TypeError');
    assert_throws(new TypeError(), () => rs.getReader(), 'getting a third reader should throw a TypeError');

  }, 'should not be able to obtain additional readers if we don\'t release the first lock');

  promise_test(t => {

    const rs = factory();
    const cancelPromise1 = rs.cancel();
    const cancelPromise2 = rs.cancel();

    assert_not_equals(cancelPromise1, cancelPromise2, 'cancel() calls should return distinct promises');

    return Promise.all([
      promise_rejects(t, error, cancelPromise1),
      promise_rejects(t, error, cancelPromise2)
    ]);

  }, 'cancel() should return a distinct rejected promise each time');

  promise_test(t => {

    const rs = factory();
    const reader = rs.getReader();
    const cancelPromise1 = reader.cancel();
    const cancelPromise2 = reader.cancel();

    assert_not_equals(cancelPromise1, cancelPromise2, 'cancel() calls should return distinct promises');

    return Promise.all([
      promise_rejects(t, error, cancelPromise1),
      promise_rejects(t, error, cancelPromise2)
    ]);

  }, 'reader cancel() should return a distinct rejected promise each time');
};

self.templatedRSEmptyReader = (label, factory) => {
  test(() => {}, 'Running templatedRSEmptyReader with ' + label);

  test(() => {

    const reader = factory().reader;

    assert_true('closed' in reader, 'has a closed property');
    assert_equals(typeof reader.closed.then, 'function', 'closed property is thenable');

    assert_equals(typeof reader.cancel, 'function', 'has a cancel method');
    assert_equals(typeof reader.read, 'function', 'has a read method');
    assert_equals(typeof reader.releaseLock, 'function', 'has a releaseLock method');

  }, 'instances have the correct methods and properties');

  test(() => {

    const stream = factory().stream;

    assert_true(stream.locked, 'locked getter should return true');

  }, 'locked should be true');

  promise_test(t => {

    const reader = factory().reader;

    reader.read().then(
      t.unreached_func('read() should not fulfill'),
      t.unreached_func('read() should not reject')
    );

    return delay(500);

  }, 'read() should never settle');

  promise_test(t => {

    const reader = factory().reader;

    reader.read().then(
      t.unreached_func('read() should not fulfill'),
      t.unreached_func('read() should not reject')
    );

    reader.read().then(
      t.unreached_func('read() should not fulfill'),
      t.unreached_func('read() should not reject')
    );

    return delay(500);

  }, 'two read()s should both never settle');

  test(() => {

    const reader = factory().reader;
    assert_not_equals(reader.read(), reader.read(), 'the promises returned should be distinct');

  }, 'read() should return distinct promises each time');

  test(() => {

    const stream = factory().stream;
    assert_throws(new TypeError(), () => stream.getReader(), 'stream.getReader() should throw a TypeError');

  }, 'getReader() again on the stream should fail');

  promise_test(t => {

    const streamAndReader = factory();
    const stream = streamAndReader.stream;
    const reader = streamAndReader.reader;

    reader.read().then(
      t.unreached_func('first read() should not fulfill'),
      t.unreached_func('first read() should not reject')
    );

    reader.read().then(
      t.unreached_func('second read() should not fulfill'),
      t.unreached_func('second read() should not reject')
    );

    reader.closed.then(
      t.unreached_func('closed should not fulfill'),
      t.unreached_func('closed should not reject')
    );

    assert_throws(new TypeError(), () => reader.releaseLock(), 'releaseLock should throw a TypeError');

    assert_true(stream.locked, 'the stream should still be locked');

    return delay(500);

  }, 'releasing the lock with pending read requests should throw but the read requests should stay pending');

  promise_test(t => {

    const reader = factory().reader;
    reader.releaseLock();

    return Promise.all([
      promise_rejects(t, new TypeError(), reader.read()),
      promise_rejects(t, new TypeError(), reader.read())
    ]);

  }, 'releasing the lock should cause further read() calls to reject with a TypeError');

  promise_test(t => {

    const reader = factory().reader;

    const closedBefore = reader.closed;
    reader.releaseLock();
    const closedAfter = reader.closed;

    assert_equals(closedBefore, closedAfter, 'the closed promise should not change identity');

    return promise_rejects(t, new TypeError(), closedBefore);

  }, 'releasing the lock should cause closed calls to reject with a TypeError');

  test(() => {

    const streamAndReader = factory();
    const stream = streamAndReader.stream;
    const reader = streamAndReader.reader;

    reader.releaseLock();
    assert_false(stream.locked, 'locked getter should return false');

  }, 'releasing the lock should cause locked to become false');

  promise_test(() => {

    const reader = factory().reader;
    reader.cancel();

    return reader.read().then(r => {
      assert_object_equals(r, { value: undefined, done: true }, 'read()ing from the reader should give a done result');
    });

  }, 'canceling via the reader should cause the reader to act closed');

  promise_test(t => {

    const stream = factory().stream;
    return promise_rejects(t, new TypeError(), stream.cancel());

  }, 'canceling via the stream should fail');
};

self.templatedRSClosedReader = (label, factory) => {
  test(() => {}, 'Running templatedRSClosedReader with ' + label);

  promise_test(() => {

    const reader = factory().reader;

    return reader.read().then(v => {
      assert_object_equals(v, { value: undefined, done: true }, 'read() should fulfill correctly');
    });

  }, 'read() should fulfill with { value: undefined, done: true }');

  promise_test(() => {

    const reader = factory().reader;

    return Promise.all([
      reader.read().then(v => {
        assert_object_equals(v, { value: undefined, done: true }, 'read() should fulfill correctly');
      }),
      reader.read().then(v => {
        assert_object_equals(v, { value: undefined, done: true }, 'read() should fulfill correctly');
      })
    ]);

  }, 'read() multiple times should fulfill with { value: undefined, done: true }');

  promise_test(() => {

    const reader = factory().reader;

    return reader.read().then(() => reader.read()).then(v => {
      assert_object_equals(v, { value: undefined, done: true }, 'read() should fulfill correctly');
    });

  }, 'read() should work when used within another read() fulfill callback');

  promise_test(() => {

    const reader = factory().reader;

    return reader.closed.then(v => assert_equals(v, undefined, 'reader closed should fulfill with undefined'));

  }, 'closed should fulfill with undefined');

  promise_test(t => {

    const reader = factory().reader;

    const closedBefore = reader.closed;
    reader.releaseLock();
    const closedAfter = reader.closed;

    assert_not_equals(closedBefore, closedAfter, 'the closed promise should change identity');

    return Promise.all([
      closedBefore.then(v => assert_equals(v, undefined, 'reader.closed acquired before release should fulfill')),
      promise_rejects(t, new TypeError(), closedAfter)
    ]);

  }, 'releasing the lock should cause closed to reject and change identity');

  promise_test(() => {

    const reader = factory().reader;
    const cancelPromise1 = reader.cancel();
    const cancelPromise2 = reader.cancel();
    const closedReaderPromise = reader.closed;

    assert_not_equals(cancelPromise1, cancelPromise2, 'cancel() calls should return distinct promises');
    assert_not_equals(cancelPromise1, closedReaderPromise, 'cancel() promise 1 should be distinct from reader.closed');
    assert_not_equals(cancelPromise2, closedReaderPromise, 'cancel() promise 2 should be distinct from reader.closed');

    return Promise.all([
      cancelPromise1.then(v => assert_equals(v, undefined, 'first cancel() should fulfill with undefined')),
      cancelPromise2.then(v => assert_equals(v, undefined, 'second cancel() should fulfill with undefined'))
    ]);

  }, 'cancel() should return a distinct fulfilled promise each time');
};

self.templatedRSErroredReader = (label, factory, error) => {
  test(() => {}, 'Running templatedRSErroredReader with ' + label);

  promise_test(t => {

    const reader = factory().reader;
    return promise_rejects(t, error, reader.closed);

  }, 'closed should reject with the error');

  promise_test(t => {

    const reader = factory().reader;
    const closedBefore = reader.closed;

    return promise_rejects(t, error, closedBefore).then(() => {
      reader.releaseLock();

      const closedAfter = reader.closed;
      assert_not_equals(closedBefore, closedAfter, 'the closed promise should change identity');

      return promise_rejects(t, new TypeError(), closedAfter);
    });

  }, 'releasing the lock should cause closed to reject and change identity');

  promise_test(t => {

    const reader = factory().reader;
    return promise_rejects(t, error, reader.read());

  }, 'read() should reject with the error');
};

self.templatedRSTwoChunksOpenReader = (label, factory, chunks) => {
  test(() => {}, 'Running templatedRSTwoChunksOpenReader with ' + label);

  promise_test(() => {

    const reader = factory().reader;

    return Promise.all([
      reader.read().then(r => {
        assert_object_equals(r, { value: chunks[0], done: false }, 'first result should be correct');
      }),
      reader.read().then(r => {
        assert_object_equals(r, { value: chunks[1], done: false }, 'second result should be correct');
      })
    ]);

  }, 'calling read() twice without waiting will eventually give both chunks (sequential)');

  promise_test(() => {

    const reader = factory().reader;

    return reader.read().then(r => {
      assert_object_equals(r, { value: chunks[0], done: false }, 'first result should be correct');

      return reader.read().then(r2 => {
        assert_object_equals(r2, { value: chunks[1], done: false }, 'second result should be correct');
      });
    });

  }, 'calling read() twice without waiting will eventually give both chunks (nested)');

  test(() => {

    const reader = factory().reader;
    assert_not_equals(reader.read(), reader.read(), 'the promises returned should be distinct');

  }, 'read() should return distinct promises each time');

  promise_test(() => {

    const reader = factory().reader;

    const promise1 = reader.closed.then(v => {
      assert_equals(v, undefined, 'reader closed should fulfill with undefined');
    });

    const promise2 = reader.read().then(r => {
      assert_object_equals(r, { value: chunks[0], done: false },
                           'promise returned before cancellation should fulfill with a chunk');
    });

    reader.cancel();

    const promise3 = reader.read().then(r => {
      assert_object_equals(r, { value: undefined, done: true },
                           'promise returned after cancellation should fulfill with an end-of-stream signal');
    });

    return Promise.all([promise1, promise2, promise3]);

  }, 'cancel() after a read() should still give that single read result');
};

self.templatedRSTwoChunksClosedReader = function (label, factory, chunks) {
  test(() => {}, 'Running templatedRSTwoChunksClosedReader with ' + label);

  promise_test(() => {

    const reader = factory().reader;

    return Promise.all([
      reader.read().then(r => {
        assert_object_equals(r, { value: chunks[0], done: false }, 'first result should be correct');
      }),
      reader.read().then(r => {
        assert_object_equals(r, { value: chunks[1], done: false }, 'second result should be correct');
      }),
      reader.read().then(r => {
        assert_object_equals(r, { value: undefined, done: true }, 'third result should be correct');
      })
    ]);

  }, 'third read(), without waiting, should give { value: undefined, done: true } (sequential)');

  promise_test(() => {

    const reader = factory().reader;

    return reader.read().then(r => {
      assert_object_equals(r, { value: chunks[0], done: false }, 'first result should be correct');

      return reader.read().then(r2 => {
        assert_object_equals(r2, { value: chunks[1], done: false }, 'second result should be correct');

        return reader.read().then(r3 => {
          assert_object_equals(r3, { value: undefined, done: true }, 'third result should be correct');
        });
      });
    });

  }, 'third read(), without waiting, should give { value: undefined, done: true } (nested)');

  promise_test(() => {

    const streamAndReader = factory();
    const stream = streamAndReader.stream;
    const reader = streamAndReader.reader;

    assert_true(stream.locked, 'stream should start locked');

    const promise = reader.closed.then(v => {
      assert_equals(v, undefined, 'reader closed should fulfill with undefined');
      assert_true(stream.locked, 'stream should remain locked');
    });

    reader.read();
    reader.read();

    return promise;

  }, 'draining the stream via read() should cause the reader closed promise to fulfill, but locked stays true');

  promise_test(() => {

    const streamAndReader = factory();
    const stream = streamAndReader.stream;
    const reader = streamAndReader.reader;

    const promise = reader.closed.then(() => {
      assert_true(stream.locked, 'the stream should start locked');
      reader.releaseLock(); // Releasing the lock after reader closed should not throw.
      assert_false(stream.locked, 'the stream should end unlocked');
    });

    reader.read();
    reader.read();

    return promise;

  }, 'releasing the lock after the stream is closed should cause locked to become false');

  promise_test(t => {

    const reader = factory().reader;

    reader.releaseLock();

    return Promise.all([
      promise_rejects(t, new TypeError(), reader.read()),
      promise_rejects(t, new TypeError(), reader.read()),
      promise_rejects(t, new TypeError(), reader.read())
    ]);

  }, 'releasing the lock should cause further read() calls to reject with a TypeError');

  promise_test(() => {

    const streamAndReader = factory();
    const stream = streamAndReader.stream;
    const reader = streamAndReader.reader;

    const readerClosed = reader.closed;

    assert_equals(reader.closed, readerClosed, 'accessing reader.closed twice in succession gives the same value');

    const promise = reader.read().then(() => {
      assert_equals(reader.closed, readerClosed, 'reader.closed is the same after read() fulfills');

      reader.releaseLock();

      assert_equals(reader.closed, readerClosed, 'reader.closed is the same after releasing the lock');

      const newReader = stream.getReader();
      return newReader.read();
    });

    assert_equals(reader.closed, readerClosed, 'reader.closed is the same after calling read()');

    return promise;

  }, 'reader\'s closed property always returns the same promise');
};
