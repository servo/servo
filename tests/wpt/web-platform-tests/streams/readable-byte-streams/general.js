'use strict';

if (self.importScripts) {
  self.importScripts('../resources/rs-utils.js');
  self.importScripts('/resources/testharness.js');
}

const error1 = new Error('error1');
error1.name = 'error1';

test(() => {
  assert_throws(new TypeError(), () => new ReadableStream().getReader({ mode: 'byob' }));
}, 'getReader({mode: "byob"}) throws on non-bytes streams');


test(() => {
  // Constructing ReadableStream with an empty underlying byte source object as parameter shouldn't throw.
  new ReadableStream({ type: 'bytes' });
}, 'ReadableStream with byte source can be constructed with no errors');

promise_test(() => {
  let startCalled = false;
  let controller;

  let resolveTestPromise;
  const testPromise = new Promise(resolve => {
    resolveTestPromise = resolve;
  });

  new ReadableStream({
    start(c) {
      controller = c;
      startCalled = true;
    },
    pull() {
      assert_true(startCalled, 'start has been called');
      assert_equals(controller.desiredSize, 256, 'desiredSize');
      resolveTestPromise();
    },
    type: 'bytes'
  }, {
    highWaterMark: 256
  });

  return testPromise;

}, 'ReadableStream with byte source: Construct and expect start and pull being called');

promise_test(() => {
  let pullCount = 0;
  let checkedNoPull = false;

  let resolveTestPromise;
  const testPromise = new Promise(resolve => {
    resolveTestPromise = resolve;
  });
  let resolveStartPromise;

  new ReadableStream({
    start() {
      return new Promise(resolve => {
        resolveStartPromise = resolve;
      });
    },
    pull() {
      if (checkedNoPull) {
        resolveTestPromise();
      }

      ++pullCount;
    },
    type: 'bytes'
  }, {
    highWaterMark: 256
  });

  Promise.resolve().then(() => {
    assert_equals(pullCount, 0);
    checkedNoPull = true;
    resolveStartPromise();
  });

  return testPromise;

}, 'ReadableStream with byte source: No automatic pull call if start doesn\'t finish');

promise_test(() => {
  new ReadableStream({
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  }, {
    highWaterMark: 0
  });

  return Promise.resolve().then(() => {});
}, 'ReadableStream with byte source: Construct with highWaterMark of 0');

promise_test(t => {
  const stream = new ReadableStream({
    type: 'bytes'
  });

  const reader = stream.getReader();
  reader.releaseLock();

  return promise_rejects(t, new TypeError(), reader.closed, 'closed must reject');
}, 'ReadableStream with byte source: getReader(), then releaseLock()');

promise_test(t => {
  const stream = new ReadableStream({
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });
  reader.releaseLock();

  return promise_rejects(t, new TypeError(), reader.closed, 'closed must reject');
}, 'ReadableStream with byte source: getReader() with mode set to byob, then releaseLock()');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      c.close();
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return reader.closed.then(() => {
    assert_throws(new TypeError(), () => stream.getReader(), 'getReader() must throw');
  });
}, 'ReadableStream with byte source: Test that closing a stream does not release a reader automatically');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      c.close();
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.closed.then(() => {
    assert_throws(new TypeError(), () => stream.getReader({ mode: 'byob' }), 'getReader() must throw');
  });
}, 'ReadableStream with byte source: Test that closing a stream does not release a BYOB reader automatically');

promise_test(t => {
  const stream = new ReadableStream({
    start(c) {
      c.error(error1);
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return promise_rejects(t, error1, reader.closed, 'closed must reject').then(() => {
    assert_throws(new TypeError(), () => stream.getReader(), 'getReader() must throw');
  });
}, 'ReadableStream with byte source: Test that erroring a stream does not release a reader automatically');

promise_test(t => {
  const stream = new ReadableStream({
    start(c) {
      c.error(error1);
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, error1, reader.closed, 'closed must reject').then(() => {
    assert_throws(new TypeError(), () => stream.getReader({ mode: 'byob' }), 'getReader() must throw');
  });
}, 'ReadableStream with byte source: Test that erroring a stream does not release a BYOB reader automatically');

test(() => {
  const stream = new ReadableStream({
    type: 'bytes'
  });

  const reader = stream.getReader();
  reader.read();
  assert_throws(new TypeError(), () => reader.releaseLock(), 'reader.releaseLock() must throw');
}, 'ReadableStream with byte source: releaseLock() on ReadableStreamReader with pending read() must throw');

promise_test(() => {
  let pullCount = 0;

  const stream = new ReadableStream({
    pull() {
      ++pullCount;
    },
    type: 'bytes'
  }, {
    highWaterMark: 8
  });

  stream.getReader();

  assert_equals(pullCount, 0, 'No pull as start() just finished and is not yet reflected to the state of the stream');

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 1, 'pull must be invoked');
  });
}, 'ReadableStream with byte source: Automatic pull() after start()');

promise_test(() => {
  let pullCount = 0;

  const stream = new ReadableStream({
    pull() {
      ++pullCount;
    },
    type: 'bytes'
  }, {
    highWaterMark: 0
  });

  const reader = stream.getReader();
  reader.read();

  assert_equals(pullCount, 0, 'No pull as start() just finished and is not yet reflected to the state of the stream');

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 1, 'pull must be invoked');
  });
}, 'ReadableStream with byte source: Automatic pull() after start() and read()');

promise_test(() => {
  let pullCount = 0;
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount === 0) {
        const byobRequest = controller.byobRequest;
        assert_not_equals(byobRequest, undefined, 'byobRequest must not be undefined');

        const view = byobRequest.view;
        assert_not_equals(view, undefined, 'byobRequest.view must no be undefined');
        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 16);
        assert_equals(view.byteOffset, 0);
        assert_equals(view.byteLength, 16);

        view[0] = 0x01;
        byobRequest.respond(1);
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes',
    autoAllocateChunkSize: 16
  }, {
    highWaterMark: 0
  });

  const reader = stream.getReader();
  const readPromise = reader.read();
  const ignoredReadPromise = reader.read();

  assert_equals(pullCount, 0, 'No pull() as start() just finished and is not yet reflected to the state of the stream');

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 1, 'pull() must have been invoked once');
    return readPromise;
  }).then(result => {
    assert_not_equals(result.value, undefined);
    assert_equals(result.value.constructor, Uint8Array);
    assert_equals(result.value.buffer.byteLength, 16);
    assert_equals(result.value.byteOffset, 0);
    assert_equals(result.value.byteLength, 1);
    assert_equals(result.value[0], 0x01);
  });
}, 'ReadableStream with byte source: autoAllocateChunkSize');

promise_test(() => {
  let pullCount = 0;
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount === 0) {
        const byobRequest = controller.byobRequest;
        assert_not_equals(byobRequest, undefined, 'byobRequest must not be undefined');

        const view = byobRequest.view;
        assert_not_equals(view, undefined, 'byobRequest.view must no be undefined');
        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 16);
        assert_equals(view.byteOffset, 0);
        assert_equals(view.byteLength, 16);

        view[0] = 0x01;
        byobRequest.respond(1);
      } else if (pullCount === 1) {
        const byobRequest = controller.byobRequest;
        assert_not_equals(byobRequest, undefined, 'byobRequest must not be undefined');

        const view = byobRequest.view;
        assert_not_equals(view, undefined, 'byobRequest.view must no be undefined');
        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 32);
        assert_equals(view.byteOffset, 0);
        assert_equals(view.byteLength, 32);

        view[0] = 0x02;
        view[1] = 0x03;
        byobRequest.respond(2);
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes',
    autoAllocateChunkSize: 16
  }, {
    highWaterMark: 0
  });

  const reader = stream.getReader();
  return reader.read().then(result => {
    assert_not_equals(result.value, undefined);
    assert_equals(result.value.constructor, Uint8Array);
    assert_equals(result.value.buffer.byteLength, 16);
    assert_equals(result.value.byteOffset, 0);
    assert_equals(result.value.byteLength, 1);
    assert_equals(result.value[0], 0x01);

    reader.releaseLock();
    const byobReader = stream.getReader({ mode: 'byob' });
    return byobReader.read(new Uint8Array(32));
  }).then(result => {
    assert_not_equals(result.value, undefined);
    assert_equals(result.value.constructor, Uint8Array);
    assert_equals(result.value.buffer.byteLength, 32);
    assert_equals(result.value.byteOffset, 0);
    assert_equals(result.value.byteLength, 2);
    assert_equals(result.value[0], 0x02);
    assert_equals(result.value[1], 0x03);
  });
}, 'ReadableStream with byte source: Mix of auto allocate and BYOB');

promise_test(() => {
  let pullCount = 0;

  const stream = new ReadableStream({
    pull() {
      ++pullCount;
    },
    type: 'bytes'
  }, {
    highWaterMark: 0
  });

  const reader = stream.getReader();
  reader.read(new Uint8Array(8));

  assert_equals(pullCount, 0, 'No pull as start() just finished and is not yet reflected to the state of the stream');

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 1, 'pull must be invoked');
  });
}, 'ReadableStream with byte source: Automatic pull() after start() and read(view)');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      c.enqueue(new Uint8Array(16));
      assert_equals(c.desiredSize, -8, 'desiredSize after enqueue() in start()');

      controller = c;
    },
    pull() {
      ++pullCount;

      if (pullCount === 1) {
        assert_equals(controller.desiredSize, 8, 'desiredSize in pull()');
      }
    },
    type: 'bytes'
  }, {
    highWaterMark: 8
  });

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 0, 'No pull as the queue was filled by start()');

    const reader = stream.getReader();

    const promise = reader.read();
    assert_equals(pullCount, 1, 'The first pull() should be made on read()');

    return promise.then(result => {
      assert_equals(result.done, false, 'result.done');

      const view = result.value;
      assert_equals(view.constructor, Uint8Array, 'view.constructor');
      assert_equals(view.buffer.byteLength, 16, 'view.buffer');
      assert_equals(view.byteOffset, 0, 'view.byteOffset');
      assert_equals(view.byteLength, 16, 'view.byteLength');
    });
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then read()');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  const promise = reader.read().then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.constructor, Uint8Array);
    assert_equals(view.buffer.byteLength, 1);
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 1);
  });

  controller.enqueue(new Uint8Array(1));

  return promise;
}, 'ReadableStream with byte source: Push source that doesn\'t understand pull signal');

promise_test(t => {
  const stream = new ReadableStream({
    pull: 'foo',
    type: 'bytes'
  });

  const reader = stream.getReader();

  return promise_rejects(t, new TypeError(), reader.read(), 'read() must fail');
}, 'ReadableStream with byte source: read(), but pull() function is not callable');

promise_test(t => {
  const stream = new ReadableStream({
    pull: 'foo',
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, new TypeError(), reader.read(new Uint8Array(1)), 'read() must fail');
}, 'ReadableStream with byte source: read(view), but pull() function is not callable');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      c.enqueue(new Uint16Array(16));
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return reader.read().then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.constructor, Uint8Array);
    assert_equals(view.buffer.byteLength, 32);
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 32);
  });
}, 'ReadableStream with byte source: enqueue() with Uint16Array, getReader(), then read()');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(16);
      view[0] = 0x01;
      view[8] = 0x02;
      c.enqueue(view);
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const byobReader = stream.getReader({ mode: 'byob' });

  return byobReader.read(new Uint8Array(8)).then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.constructor, Uint8Array, 'value.constructor');
    assert_equals(view.buffer.byteLength, 8, 'value.buffer.byteLength');
    assert_equals(view.byteOffset, 0, 'value.byteOffset');
    assert_equals(view.byteLength, 8, 'value.byteLength');
    assert_equals(view[0], 0x01);

    byobReader.releaseLock();

    const reader = stream.getReader();

    return reader.read();
  }).then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.constructor, Uint8Array, 'value.constructor');
    assert_equals(view.buffer.byteLength, 16, 'value.buffer.byteLength');
    assert_equals(view.byteOffset, 8, 'value.byteOffset');
    assert_equals(view.byteLength, 8, 'value.byteLength');
    assert_equals(view[0], 0x02);
  });
}, 'ReadableStream with byte source: enqueue(), read(view) partially, then read()');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  controller.enqueue(new Uint8Array(16));
  controller.close();

  return reader.read().then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0, 'byteOffset');
    assert_equals(view.byteLength, 16, 'byteLength');

    return reader.read();
  }).then(result => {
    assert_equals(result.done, true, 'done');
    assert_equals(result.value, undefined, 'value');
  });
}, 'ReadableStream with byte source: getReader(), enqueue(), close(), then read()');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      c.enqueue(new Uint8Array(16));
      c.close();
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return reader.read().then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0, 'byteOffset');
    assert_equals(view.byteLength, 16, 'byteLength');

    return reader.read();
  }).then(result => {
    assert_equals(result.done, true, 'done');
    assert_equals(result.value, undefined, 'value');
  });
}, 'ReadableStream with byte source: enqueue(), close(), getReader(), then read()');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      controller.enqueue(new Uint8Array(16));
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return reader.read().then(result => {
    assert_equals(result.done, false, 'done');
    assert_equals(result.value.byteLength, 16, 'byteLength');
  });
}, 'ReadableStream with byte source: Respond to pull() by enqueue()');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest is undefined');

      if (pullCount === 0) {
        assert_equals(controller.desiredSize, 256, 'desiredSize on pull');

        controller.enqueue(new Uint8Array(1));
        assert_equals(controller.desiredSize, 256, 'desiredSize after 1st enqueue()');

        controller.enqueue(new Uint8Array(1));
        assert_equals(controller.desiredSize, 256, 'desiredSize after 2nd enqueue()');
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  }, {
    highWaterMark: 256
  });

  const reader = stream.getReader();

  const p0 = reader.read();
  const p1 = reader.read();
  const p2 = reader.read();

  // Respond to the first pull call.
  controller.enqueue(new Uint8Array(1));

  assert_equals(pullCount, 0, 'pullCount after the enqueue() outside pull');

  return Promise.all([p0, p1, p2]).then(result => {
    assert_equals(pullCount, 1, 'pullCount after completion of all read()s');

    assert_equals(result[0].done, false, 'result[0].done');
    assert_equals(result[0].value.byteLength, 1, 'result[0].value.byteLength');
    assert_equals(result[1].done, false, 'result[1].done');
    assert_equals(result[1].value.byteLength, 1, 'result[1].value.byteLength');
    assert_equals(result[2].done, false, 'result[2].done');
    assert_equals(result[2].value.byteLength, 1, 'result[2].value.byteLength');
  });
}, 'ReadableStream with byte source: Respond to pull() by enqueue() asynchronously');

promise_test(() => {
  let controller;

  let pullCount = 0;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount === 0) {
        assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined before respond()');

        const view = controller.byobRequest.view;
        view[0] = 0x01;
        controller.byobRequest.respond(1);

        assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined after respond()');
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(1)).then(result => {
    assert_equals(result.done, false, 'result.done');
    assert_equals(result.value.byteLength, 1, 'result.value.byteLength');
    assert_equals(result.value[0], 0x01, 'result.value[0]');
  });
}, 'ReadableStream with byte source: read(view), then respond()');

promise_test(() => {
  let controller;

  let pullCount = 0;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount === 0) {
        assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined before respond()');

        // Emulate ArrayBuffer transfer by just creating a new ArrayBuffer and pass it. By checking the result of
        // read(view), we test that the respond()'s buffer argument is working correctly.
        //
        // A real implementation of the underlying byte source would transfer controller.byobRequest.view.buffer into
        // a new ArrayBuffer, then construct a view around it and write to it.
        const transferredView = new Uint8Array(1);
        transferredView[0] = 0x01;
        controller.byobRequest.respondWithNewView(transferredView);

        assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined after respond()');
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(1)).then(result => {
    assert_equals(result.done, false, 'result.done');
    assert_equals(result.value.byteLength, 1, 'result.value.byteLength');
    assert_equals(result.value[0], 0x01, 'result.value[0]');
  });
}, 'ReadableStream with byte source: read(view), then respond() with a transferred ArrayBuffer');

promise_test(t => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest is not undefined');

      assert_throws(new RangeError(), () => controller.byobRequest.respond(2), 'respond() must throw');
      controller.byobRequest.respond(1);
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(1)).catch(e => {
    assert_unreached(e);
    t.done();
  });
}, 'ReadableStream with byte source: read(view), then respond() with too big value');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount > 1) {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;

      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');
      const view = controller.byobRequest.view;

      assert_equals(view.constructor, Uint8Array);
      assert_equals(view.buffer.byteLength, 4);

      assert_equals(view.byteOffset, 0);
      assert_equals(view.byteLength, 4);

      view[0] = 0x01;
      view[1] = 0x02;
      view[2] = 0x03;

      controller.byobRequest.respond(3);
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint16Array(2)).then(result => {
    assert_equals(pullCount, 1);

    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0, 'byteOffset');
    assert_equals(view.byteLength, 2, 'byteLength');

    assert_equals(view[0], 0x0201);

    return reader.read(new Uint8Array(1));
  }).then(result => {
    assert_equals(pullCount, 1);

    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0, 'byteOffset');
    assert_equals(view.byteLength, 1, 'byteLength');

    assert_equals(view[0], 0x03);
  });
}, 'ReadableStream with byte source: respond(3) to read(view) with 2 element Uint16Array enqueues the 1 byte ' +
   'remainder');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(16);
      view[15] = 0x01;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest is undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(16)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 16);
    assert_equals(view[15], 0x01);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then read(view)');

promise_test(() => {
  let cancelCount = 0;

  const passedReason = new TypeError('foo');

  const stream = new ReadableStream({
    start(c) {
      c.enqueue(new Uint8Array(16));
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    cancel(reason) {
      if (cancelCount === 0) {
        assert_equals(reason, passedReason);
      } else {
        assert_unreached('Too many cancel calls');
      }

      ++cancelCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return reader.cancel(passedReason).then(result => {
    assert_equals(result, undefined);
    assert_equals(cancelCount, 1);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then cancel() (mode = not BYOB)');

promise_test(() => {
  let cancelCount = 0;

  const passedReason = new TypeError('foo');

  const stream = new ReadableStream({
    start(c) {
      c.enqueue(new Uint8Array(16));
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    cancel(reason) {
      if (cancelCount === 0) {
        assert_equals(reason, passedReason);
      } else {
        assert_unreached('Too many cancel calls');
      }

      ++cancelCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.cancel(passedReason).then(result => {
    assert_equals(result, undefined);
    assert_equals(cancelCount, 1);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then cancel() (mode = BYOB)');

promise_test(() => {
  let cancelCount = 0;

  const passedReason = new TypeError('foo');

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    cancel(reason) {
      if (cancelCount === 0) {
        assert_equals(reason, passedReason);

        controller.byobRequest.respond(0);
      } else {
        assert_unreached('Too many cancel calls');
      }

      ++cancelCount;

      return 'bar';
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const readPromise0 = reader.read(new Uint8Array(1)).then(result => {
    assert_equals(result.done, true);
  });

  const readPromise1 = reader.cancel(passedReason).then(result => {
    assert_equals(result, undefined);
    assert_equals(cancelCount, 1);
  });

  return Promise.all([readPromise0, readPromise1]);
}, 'ReadableStream with byte source: getReader(), read(view), then cancel()');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest is undefined');

      if (pullCount === 0) {
        assert_equals(controller.byobRequest.view.byteLength, 2, 'byteLength before enqueue()');
        controller.enqueue(new Uint8Array(1));
        assert_equals(controller.byobRequest.view.byteLength, 1, 'byteLength after enqueue()');
      } else {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  return Promise.resolve().then(() => {
    assert_equals(pullCount, 0, 'No pull() as no read(view) yet');

    const reader = stream.getReader({ mode: 'byob' });

    const promise = reader.read(new Uint16Array(1)).then(result => {
      assert_equals(result.done, true, 'result.done');
      assert_equals(result.value.constructor, Uint16Array, 'result.value');
    });

    assert_equals(pullCount, 1, '1 pull() should have been made in response to partial fill by enqueue()');

    reader.cancel();

    // Tell that the buffer given via pull() is returned.
    controller.byobRequest.respond(0);

    return promise;
  });
}, 'ReadableStream with byte source: cancel() with partially filled pending pull() request');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(8);
      view[7] = 0x01;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const buffer = new ArrayBuffer(16);

  return reader.read(new Uint8Array(buffer, 8, 8)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.constructor, Uint8Array);
    assert_equals(view.buffer.byteLength, 16);
    assert_equals(view.byteOffset, 8);
    assert_equals(view.byteLength, 8);
    assert_equals(view[7], 0x01);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then read(view) where view.buffer is not fully ' +
   'covered by view');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      let view;

      view = new Uint8Array(16);
      view[15] = 123;
      c.enqueue(view);

      view = new Uint8Array(8);
      view[7] = 111;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(24)).then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0, 'byteOffset');
    assert_equals(view.byteLength, 24, 'byteLength');
    assert_equals(view[15], 123, 'Contents are set from the first chunk');
    assert_equals(view[23], 111, 'Contents are set from the second chunk');
  });
}, 'ReadableStream with byte source: Multiple enqueue(), getReader(), then read(view)');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(16);
      view[15] = 0x01;
      c.enqueue(view);
    },
    pull(controller) {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(24)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 16);
    assert_equals(view[15], 0x01);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then read(view) with a bigger view');

promise_test(() => {
  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(16);
      view[7] = 0x01;
      view[15] = 0x02;
      c.enqueue(view);
    },
    pull(controller) {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(8)).then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 8);
    assert_equals(view[7], 0x01);

    return reader.read(new Uint8Array(8));
  }).then(result => {
    assert_equals(result.done, false, 'done');

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 8);
    assert_equals(view[7], 0x02);
  });
}, 'ReadableStream with byte source: enqueue(), getReader(), then read(view) with a smaller views');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(1);
      view[0] = 0xff;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      if (controller.byobRequest === undefined) {
        return;
      }

      const view = controller.byobRequest.view;

      assert_equals(view.constructor, Uint8Array);
      assert_equals(view.buffer.byteLength, 2);

      assert_equals(view.byteOffset, 1);
      assert_equals(view.byteLength, 1);

      view[0] = 0xaa;
      controller.byobRequest.respond(1);
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint16Array(1)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 2);
    assert_equals(view[0], 0xaaff);
  });
}, 'ReadableStream with byte source: enqueue() 1 byte, getReader(), then read(view) with Uint16Array');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(3);
      view[0] = 0x01;
      view[2] = 0x02;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');

      if (pullCount === 0) {
        const view = controller.byobRequest.view;

        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 2);
        assert_equals(view.byteOffset, 1);
        assert_equals(view.byteLength, 1);

        view[0] = 0x03;
        controller.byobRequest.respond(1);

        assert_equals(controller.desiredSize, 0, 'desiredSize');
      } else {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  // Wait for completion of the start method to be reflected.
  return Promise.resolve().then(() => {
    const reader = stream.getReader({ mode: 'byob' });

    const promise = reader.read(new Uint16Array(2)).then(result => {
      assert_equals(result.done, false, 'done');

      const view = result.value;
      assert_equals(view.constructor, Uint16Array, 'constructor');
      assert_equals(view.buffer.byteLength, 4, 'buffer.byteLength');
      assert_equals(view.byteOffset, 0, 'byteOffset');
      assert_equals(view.byteLength, 2, 'byteLength');
      assert_equals(view[0], 0x0001, 'Contents are set');

      const p = reader.read(new Uint16Array(1));

      assert_equals(pullCount, 1);

      return p;
    }).then(result => {
      assert_equals(result.done, false, 'done');

      const view = result.value;
      assert_equals(view.buffer.byteLength, 2, 'buffer.byteLength');
      assert_equals(view.byteOffset, 0, 'byteOffset');
      assert_equals(view.byteLength, 2, 'byteLength');
      assert_equals(view[0], 0x0302, 'Contents are set');
    });

    assert_equals(pullCount, 0);

    return promise;
  });
}, 'ReadableStream with byte source: enqueue() 3 byte, getReader(), then read(view) with 2-element Uint16Array');

promise_test(t => {
  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(1);
      view[0] = 0xff;
      c.enqueue(view);
      c.close();
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });


  const promise = promise_rejects(t, new TypeError(), reader.read(new Uint16Array(1)), 'read(view) must fail');
  return promise_rejects(t, new TypeError(), promise.then(() => reader.closed));
}, 'ReadableStream with byte source: read(view) with Uint16Array on close()-d stream with 1 byte enqueue()-d must ' +
   'fail');

promise_test(t => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      const view = new Uint8Array(1);
      view[0] = 0xff;
      c.enqueue(view);

      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');

      if (pullCount === 0) {
        const view = controller.byobRequest.view;

        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 2);

        assert_equals(view.byteOffset, 1);
        assert_equals(view.byteLength, 1);
      } else {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  let promise = promise_rejects(t, new TypeError(), reader.read(new Uint16Array(1)), 'read(view) must fail');
  promise = promise_rejects(t, new TypeError(), promise.then(() => reader.closed));
  promise = promise.then(() => {
    assert_equals(pullCount, 0);
  });

  assert_throws(new TypeError(), () => controller.close(), 'controller.close() must throw');

  return promise;
}, 'ReadableStream with byte source: A stream must be errored if close()-d before fulfilling read(view) with ' +
   'Uint16Array');

test(() => {
  let controller;

  new ReadableStream({
    start(c) {
      controller = c;
    },
    type: 'bytes'
  });

  // Enqueue a chunk so that the stream doesn't get closed. This is to check duplicate close() calls are rejected
  // even if the stream has not yet entered the closed state.
  const view = new Uint8Array(1);
  controller.enqueue(view);
  controller.close();

  assert_throws(new TypeError(), () => controller.close(), 'controller.close() must throw');
}, 'ReadableStream with byte source: Throw if close()-ed more than once');

test(() => {
  let controller;

  new ReadableStream({
    start(c) {
      controller = c;
    },
    type: 'bytes'
  });

  // Enqueue a chunk so that the stream doesn't get closed. This is to check enqueue() after close() is  rejected
  // even if the stream has not yet entered the closed state.
  const view = new Uint8Array(1);
  controller.enqueue(view);
  controller.close();

  assert_throws(new TypeError(), () => controller.enqueue(view), 'controller.close() must throw');
}, 'ReadableStream with byte source: Throw on enqueue() after close()');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');
      const view = controller.byobRequest.view;

      assert_equals(view.constructor, Uint8Array);
      assert_equals(view.buffer.byteLength, 16);

      assert_equals(view.byteOffset, 0);
      assert_equals(view.byteLength, 16);

      view[15] = 0x01;
      controller.byobRequest.respond(16);
      controller.close();
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint8Array(16)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 16);
    assert_equals(view[15], 0x01);

    return reader.read(new Uint8Array(16));
  }).then(result => {
    assert_equals(result.done, true);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 0);
  });
}, 'ReadableStream with byte source: read(view), then respond() and close() in pull()');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (controller.byobRequest === undefined) {
        return;
      }

      if (pullCount < 1) {
        for (let i = 0; i < 4; ++i) {
          const view = controller.byobRequest.view;

          assert_equals(view.constructor, Uint8Array);
          assert_equals(view.buffer.byteLength, 4);

          assert_equals(view.byteOffset, i);
          assert_equals(view.byteLength, 4 - i);

          view[0] = 0x01;
          controller.byobRequest.respond(1);
        }
      } else {
        assert_unreached('Too many pull() calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return reader.read(new Uint32Array(1)).then(result => {
    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 4);
    assert_equals(view[0], 0x01010101);
  });
}, 'ReadableStream with byte source: read(view) with Uint32Array, then fill it by multiple respond() calls');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');

      if (pullCount > 1) {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  const p0 = reader.read().then(result => {
    assert_equals(pullCount, 1);

    controller.enqueue(new Uint8Array(2));

    // Since the queue has data no less than HWM, no more pull.
    assert_equals(pullCount, 1);

    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.constructor, Uint8Array);
    assert_equals(view.buffer.byteLength, 1);
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 1);
  });

  assert_equals(pullCount, 0, 'No pull should have been made since the startPromise has not yet been handled');

  const p1 = reader.read().then(result => {
    assert_equals(pullCount, 1);

    assert_equals(result.done, false);

    const view = result.value;
    assert_equals(view.constructor, Uint8Array);
    assert_equals(view.buffer.byteLength, 2);
    assert_equals(view.byteOffset, 0);
    assert_equals(view.byteLength, 2);
  });

  assert_equals(pullCount, 0, 'No pull should have been made since the startPromise has not yet been handled');

  controller.enqueue(new Uint8Array(1));

  assert_equals(pullCount, 0, 'No pull should have been made since the startPromise has not yet been handled');

  return Promise.all([p0, p1]);
}, 'ReadableStream with byte source: read() twice, then enqueue() twice');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');

      if (pullCount === 0) {
        const view = controller.byobRequest.view;

        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 16);

        assert_equals(view.byteOffset, 0);
        assert_equals(view.byteLength, 16);
      } else {
        assert_unreached('Too many pull calls');
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const p0 = reader.read(new Uint8Array(16)).then(result => {
    assert_equals(result.done, true, '1st read: done');

    const view = result.value;
    assert_equals(view.buffer.byteLength, 16, '1st read: buffer.byteLength');
    assert_equals(view.byteOffset, 0, '1st read: byteOffset');
    assert_equals(view.byteLength, 0, '1st read: byteLength');
  });

  const p1 = reader.read(new Uint8Array(32)).then(result => {
    assert_equals(result.done, true, '2nd read: done');

    const view = result.value;
    assert_equals(view.buffer.byteLength, 32, '2nd read: buffer.byteLength');
    assert_equals(view.byteOffset, 0, '2nd read: byteOffset');
    assert_equals(view.byteLength, 0, '2nd read: byteLength');
  });

  controller.close();
  controller.byobRequest.respond(0);

  return Promise.all([p0, p1]);
}, 'ReadableStream with byte source: Multiple read(view), close() and respond()');

promise_test(() => {
  let pullCount = 0;

  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      if (pullCount === 0) {
        assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');
        const view = controller.byobRequest.view;

        assert_equals(view.constructor, Uint8Array);
        assert_equals(view.buffer.byteLength, 16);

        assert_equals(view.byteOffset, 0);
        assert_equals(view.byteLength, 16);
      } else {
        assert_unreached();
      }

      ++pullCount;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const p0 = reader.read(new Uint8Array(16)).then(result => {
    assert_equals(result.done, false, '1st read: done');

    const view = result.value;
    assert_equals(view.buffer.byteLength, 16, '1st read: buffer.byteLength');
    assert_equals(view.byteOffset, 0, '1st read: byteOffset');
    assert_equals(view.byteLength, 16, '1st read: byteLength');
  });

  const p1 = reader.read(new Uint8Array(16)).then(result => {
    assert_equals(result.done, false, '2nd read: done');

    const view = result.value;
    assert_equals(view.buffer.byteLength, 16, '2nd read: buffer.byteLength');
    assert_equals(view.byteOffset, 0, '2nd read: byteOffset');
    assert_equals(view.byteLength, 8, '2nd read: byteLength');
  });

  controller.enqueue(new Uint8Array(24));

  return Promise.all([p0, p1]);
}, 'ReadableStream with byte source: Multiple read(view), big enqueue()');

promise_test(() => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  let bytesRead = 0;

  function pump() {
    return reader.read(new Uint8Array(7)).then(result => {
      if (result.done) {
        assert_equals(bytesRead, 1024);

        return null;
      }

      bytesRead += result.value.byteLength;

      return pump();
    }).catch(e => {
      assert_unreached(e);
    });
  }
  const promise = pump();

  controller.enqueue(new Uint8Array(512));
  controller.enqueue(new Uint8Array(512));
  controller.close();

  return promise;
}, 'ReadableStream with byte source: Multiple read(view) and multiple enqueue()');

promise_test(t => {
  const stream = new ReadableStream({
    pull(controller) {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, new TypeError(), reader.read(), 'read() must fail');
}, 'ReadableStream with byte source: read(view) with passing undefined as view must fail');

promise_test(t => {
  const stream = new ReadableStream({
    pull(controller) {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, new TypeError(), reader.read(new Uint8Array(0)), 'read(view) must fail');
}, 'ReadableStream with byte source: read(view) with zero-length view must fail');

promise_test(t => {
  const stream = new ReadableStream({
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, new TypeError(), reader.read({}), 'read(view) must fail');
}, 'ReadableStream with byte source: read(view) with passing an empty object as view must fail');

promise_test(t => {
  const stream = new ReadableStream({
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, new TypeError(),
                         reader.read({ buffer: new ArrayBuffer(10), byteOffset: 0, byteLength: 10 }),
                         'read(view) must fail');
}, 'ReadableStream with byte source: Even read(view) with passing ArrayBufferView like object as view must fail');

promise_test(t => {
  const stream = new ReadableStream({
    start(c) {
      c.error(error1);
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return promise_rejects(t, error1, reader.read(), 'read() must fail');
}, 'ReadableStream with byte source: read() on an errored stream');

promise_test(t => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  const promise = promise_rejects(t, error1, reader.read(), 'read() must fail');

  controller.error(error1);

  return promise;
}, 'ReadableStream with byte source: read(), then error()');

promise_test(t => {
  const stream = new ReadableStream({
    start(c) {
      c.error(error1);
    },
    pull() {
      assert_unreached('pull must not be called');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, error1, reader.read(new Uint8Array(1)), 'read() must fail');
}, 'ReadableStream with byte source: read(view) on an errored stream');

promise_test(t => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const promise = promise_rejects(t, error1, reader.read(new Uint8Array(1)), 'read() must fail');

  controller.error(error1);

  return promise;
}, 'ReadableStream with byte source: read(view), then error()');

promise_test(t => {
  let controller;

  const testError = new TypeError('foo');

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
      throw testError;
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  const promise = promise_rejects(t, testError, reader.read(), 'read() must fail');
  return promise_rejects(t, testError, promise.then(() => reader.closed));
}, 'ReadableStream with byte source: Throwing in pull function must error the stream');

promise_test(t => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_equals(controller.byobRequest, undefined, 'byobRequest must be undefined');
      controller.error(error1);
      throw new TypeError('foo');
    },
    type: 'bytes'
  });

  const reader = stream.getReader();

  return promise_rejects(t, error1, reader.read(), 'read() must fail').then(() => {
    return promise_rejects(t, error1, reader.closed, 'closed must fail');
  });
}, 'ReadableStream with byte source: Throwing in pull in response to read() must be ignored if the stream is ' +
   'errored in it');

promise_test(t => {
  let controller;

  const testError = new TypeError('foo');

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');
      throw testError;
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  const promise = promise_rejects(t, testError, reader.read(new Uint8Array(1)), 'read(view) must fail');
  return promise_rejects(t, testError, promise.then(() => reader.closed));
}, 'ReadableStream with byte source: Throwing in pull in response to read(view) function must error the stream');

promise_test(t => {
  let controller;

  const stream = new ReadableStream({
    start(c) {
      controller = c;
    },
    pull() {
      assert_not_equals(controller.byobRequest, undefined, 'byobRequest must not be undefined');
      controller.error(error1);
      throw new TypeError('foo');
    },
    type: 'bytes'
  });

  const reader = stream.getReader({ mode: 'byob' });

  return promise_rejects(t, error1, reader.read(new Uint8Array(1)), 'read(view) must fail').then(() => {
    return promise_rejects(t, error1, reader.closed, 'closed must fail');
  });
}, 'ReadableStream with byte source: Throwing in pull in response to read(view) must be ignored if the stream is ' +
   'errored in it');


test(() => {
  const ReadableStreamBYOBReader = new ReadableStream({ type: 'bytes' }).getReader({ mode: 'byob' }).constructor;
  const stream = new ReadableStream({ type: 'bytes' });
  new ReadableStreamBYOBReader(stream);
}, 'ReadableStreamBYOBReader can be constructed directly');

test(() => {
  const ReadableStreamBYOBReader = new ReadableStream({ type: 'bytes' }).getReader({ mode: 'byob' }).constructor;
  assert_throws(new TypeError(), () => new ReadableStreamBYOBReader({}), 'constructor must throw');
}, 'ReadableStreamBYOBReader constructor requires a ReadableStream argument');

test(() => {
  const ReadableStreamBYOBReader = new ReadableStream({ type: 'bytes' }).getReader({ mode: 'byob' }).constructor;
  const stream = new ReadableStream({ type: 'bytes' });
  stream.getReader();
  assert_throws(new TypeError(), () => new ReadableStreamBYOBReader(stream), 'constructor must throw');
}, 'ReadableStreamBYOBReader constructor requires an unlocked ReadableStream');

test(() => {
  const ReadableStreamBYOBReader = new ReadableStream({ type: 'bytes' }).getReader({ mode: 'byob' }).constructor;
  const stream = new ReadableStream();
  assert_throws(new TypeError(), () => new ReadableStreamBYOBReader(stream), 'constructor must throw');
}, 'ReadableStreamBYOBReader constructor requires a ReadableStream with type "bytes"');

done();
