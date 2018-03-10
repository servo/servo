'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
}

test(() => {
  const ws = new WritableStream({});
  const writer = ws.getWriter();
  writer.releaseLock();

  assert_throws(new TypeError(), () => writer.desiredSize, 'desiredSize should throw a TypeError');
}, 'desiredSize on a released writer');

test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();

  assert_equals(writer.desiredSize, 1, 'desiredSize should be 1');
}, 'desiredSize initial value');

promise_test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();

  writer.close();

  return writer.closed.then(() => {
    assert_equals(writer.desiredSize, 0, 'desiredSize should be 0');
  });
}, 'desiredSize on a writer for a closed stream');

test(() => {
  const ws = new WritableStream({
    start(c) {
      c.error();
    }
  });

  const writer = ws.getWriter();
  assert_equals(writer.desiredSize, null, 'desiredSize should be null');
}, 'desiredSize on a writer for an errored stream');

test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();
  writer.close();
  writer.releaseLock();

  ws.getWriter();
}, 'ws.getWriter() on a closing WritableStream');

promise_test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();
  return writer.close().then(() => {
    writer.releaseLock();

    ws.getWriter();
  });
}, 'ws.getWriter() on a closed WritableStream');

test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();
  writer.abort();
  writer.releaseLock();

  ws.getWriter();
}, 'ws.getWriter() on an aborted WritableStream');

promise_test(() => {
  const ws = new WritableStream({
    start(c) {
      c.error();
    }
  });

  const writer = ws.getWriter();
  return writer.closed.then(
    v => assert_unreached('writer.closed fulfilled unexpectedly with: ' + v),
    () => {
      writer.releaseLock();

      ws.getWriter();
    }
  );
}, 'ws.getWriter() on an errored WritableStream');

promise_test(() => {
  const ws = new WritableStream({});

  const writer = ws.getWriter();
  writer.releaseLock();

  return writer.closed.then(
    v => assert_unreached('writer.closed fulfilled unexpectedly with: ' + v),
    closedRejection => {
      assert_equals(closedRejection.name, 'TypeError', 'closed promise should reject with a TypeError');
      return writer.ready.then(
        v => assert_unreached('writer.ready fulfilled unexpectedly with: ' + v),
        readyRejection => assert_equals(readyRejection, closedRejection,
          'ready promise should reject with the same error')
      );
    }
  );
}, 'closed and ready on a released writer');

promise_test(() => {
  const promises = {};
  const resolvers = {};
  for (const methodName of ['start', 'write', 'close', 'abort']) {
    promises[methodName] = new Promise(resolve => {
      resolvers[methodName] = resolve;
    });
  }

  // Calls to Sink methods after the first are implicitly ignored. Only the first value that is passed to the resolver
  // is used.
  class Sink {
    start() {
      // Called twice
      resolvers.start(this);
    }

    write() {
      resolvers.write(this);
    }

    close() {
      resolvers.close(this);
    }

    abort() {
      resolvers.abort(this);
    }
  }

  const theSink = new Sink();
  const ws = new WritableStream(theSink);

  const writer = ws.getWriter();

  writer.write('a');
  writer.close();

  const ws2 = new WritableStream(theSink);
  const writer2 = ws2.getWriter();
  writer2.abort();

  return promises.start
      .then(thisValue => assert_equals(thisValue, theSink, 'start should be called as a method'))
      .then(() => promises.write)
      .then(thisValue => assert_equals(thisValue, theSink, 'write should be called as a method'))
      .then(() => promises.close)
      .then(thisValue => assert_equals(thisValue, theSink, 'close should be called as a method'))
      .then(() => promises.abort)
      .then(thisValue => assert_equals(thisValue, theSink, 'abort should be called as a method'));
}, 'WritableStream should call underlying sink methods as methods');

promise_test(t => {
  function functionWithOverloads() {}
  functionWithOverloads.apply = t.unreached_func('apply() should not be called');
  functionWithOverloads.call = t.unreached_func('call() should not be called');
  const underlyingSink = {
    start: functionWithOverloads,
    write: functionWithOverloads,
    close: functionWithOverloads,
    abort: functionWithOverloads
  };
  // Test start(), write(), close().
  const ws1 = new WritableStream(underlyingSink);
  const writer1 = ws1.getWriter();
  writer1.write('a');
  writer1.close();

  // Test abort().
  const abortError = new Error();
  abortError.name = 'abort error';

  const ws2 = new WritableStream(underlyingSink);
  const writer2 = ws2.getWriter();
  writer2.abort(abortError);

  // Test abort() with a close underlying sink method present. (Historical; see
  // https://github.com/whatwg/streams/issues/620#issuecomment-263483953 for what used to be
  // tested here. But more coverage can't hurt.)
  const ws3 = new WritableStream({
    start: functionWithOverloads,
    write: functionWithOverloads,
    close: functionWithOverloads
  });
  const writer3 = ws3.getWriter();
  writer3.abort(abortError);

  return writer1.closed
      .then(() => promise_rejects(t, abortError, writer2.closed, 'writer2.closed should be rejected'))
      .then(() => promise_rejects(t, abortError, writer3.closed, 'writer3.closed should be rejected'));
}, 'methods should not not have .apply() or .call() called');

promise_test(() => {
  const strategy = {
    size() {
      if (this !== undefined) {
        throw new Error('size called as a method');
      }
      return 1;
    }
  };

  const ws = new WritableStream({}, strategy);
  const writer = ws.getWriter();
  return writer.write('a');
}, 'WritableStream\'s strategy.size should not be called as a method');

promise_test(() => {
  const ws = new WritableStream();
  const writer1 = ws.getWriter();
  assert_equals(undefined, writer1.releaseLock(), 'releaseLock() should return undefined');
  const writer2 = ws.getWriter();
  assert_equals(undefined, writer1.releaseLock(), 'no-op releaseLock() should return undefined');
  // Calling releaseLock() on writer1 should not interfere with writer2. If it did, then the ready promise would be
  // rejected.
  return writer2.ready;
}, 'redundant releaseLock() is no-op');

promise_test(() => {
  const events = [];
  const ws = new WritableStream();
  const writer = ws.getWriter();
  return writer.ready.then(() => {
    // Force the ready promise back to a pending state.
    const writerPromise = writer.write('dummy');
    const readyPromise = writer.ready.catch(() => events.push('ready'));
    const closedPromise = writer.closed.catch(() => events.push('closed'));
    writer.releaseLock();
    return Promise.all([readyPromise, closedPromise]).then(() => {
      assert_array_equals(events, ['ready', 'closed'], 'ready promise should fire before closed promise');
      // Stop the writer promise hanging around after the test has finished.
      return Promise.all([
        writerPromise,
        ws.abort()
      ]);
    });
  });
}, 'ready promise should fire before closed on releaseLock');

done();
