'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
  self.importScripts('../resources/test-utils.js');
  self.importScripts('../resources/rs-utils.js');
  self.importScripts('../resources/recording-streams.js');
}

const error1 = new Error('error1!');
error1.name = 'error1';

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.close();
    }
  });

  const ws = recordingWritableStream(undefined, new CountQueuingStrategy({ highWaterMark: 0 }));

  const pipePromise = rs.pipeTo(ws, { preventCancel: true });

  // Wait and make sure it doesn't do any reading.
  return flushAsyncEvents().then(() => {
    ws.controller.error(error1);
  })
  .then(() => promise_rejects(t, error1, pipePromise, 'pipeTo must reject with the same error'))
  .then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, []);
  })
  .then(() => readableStreamToArray(rs))
  .then(chunksNotPreviouslyRead => {
    assert_array_equals(chunksNotPreviouslyRead, ['a', 'b']);
  });

}, 'Piping from a non-empty ReadableStream into a WritableStream that does not desire chunks');

promise_test(() => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('b');
      controller.close();
    }
  });

  let resolveWritePromise;
  const ws = recordingWritableStream({
    write() {
      if (!resolveWritePromise) {
        // first write
        return new Promise(resolve => {
          resolveWritePromise = resolve;
        });
      }
      return undefined;
    }
  });

  const writer = ws.getWriter();
  const firstWritePromise = writer.write('a');
  assert_equals(writer.desiredSize, 0, 'after writing the writer\'s desiredSize must be 0');
  writer.releaseLock();

  // firstWritePromise won't settle until we call resolveWritePromise.

  const pipePromise = rs.pipeTo(ws);

  return flushAsyncEvents().then(() => resolveWritePromise())
    .then(() => Promise.all([firstWritePromise, pipePromise]))
    .then(() => {
      assert_array_equals(rs.eventsWithoutPulls, []);
      assert_array_equals(ws.events, ['write', 'a', 'write', 'b', 'close']);
    });

}, 'Piping from a non-empty ReadableStream into a WritableStream that does not desire chunks, but then does');

promise_test(() => {

  const rs = recordingReadableStream();

  const startPromise = Promise.resolve();
  let resolveWritePromise;
  const ws = recordingWritableStream({
    start() {
      return startPromise;
    },
    write() {
      if (!resolveWritePromise) {
        // first write
        return new Promise(resolve => {
          resolveWritePromise = resolve;
        });
      }
      return undefined;
    }
  });

  const writer = ws.getWriter();
  writer.write('a');

  return startPromise.then(() => {
    assert_array_equals(ws.events, ['write', 'a']);
    assert_equals(writer.desiredSize, 0, 'after writing the writer\'s desiredSize must be 0');
    writer.releaseLock();

    const pipePromise = rs.pipeTo(ws);

    rs.controller.enqueue('b');
    resolveWritePromise();
    rs.controller.close();

    return pipePromise.then(() => {
      assert_array_equals(rs.eventsWithoutPulls, []);
      assert_array_equals(ws.events, ['write', 'a', 'write', 'b', 'close']);
    });
  });

}, 'Piping from an empty ReadableStream into a WritableStream that does not desire chunks, but then the readable ' +
   'stream becomes non-empty and the writable stream starts desiring chunks');

promise_test(() => {
  const unreadChunks = ['b', 'c', 'd'];

  const rs = recordingReadableStream({
    pull(controller) {
      controller.enqueue(unreadChunks.shift());
      if (unreadChunks.length === 0) {
        controller.close();
      }
    }
  }, new CountQueuingStrategy({ highWaterMark: 0 }));

  let resolveWritePromise;
  const ws = recordingWritableStream({
    write() {
      if (!resolveWritePromise) {
        // first write
        return new Promise(resolve => {
          resolveWritePromise = resolve;
        });
      }
      return undefined;
    }
  }, new CountQueuingStrategy({ highWaterMark: 3 }));

  const writer = ws.getWriter();
  const firstWritePromise = writer.write('a');
  assert_equals(writer.desiredSize, 2, 'after writing the writer\'s desiredSize must be 2');
  writer.releaseLock();

  // firstWritePromise won't settle until we call resolveWritePromise.

  const pipePromise = rs.pipeTo(ws);

  return flushAsyncEvents().then(() => {
    assert_array_equals(ws.events, ['write', 'a']);
    assert_equals(unreadChunks.length, 1, 'chunks should continue to be enqueued until the HWM is reached');
  }).then(() => resolveWritePromise())
    .then(() => Promise.all([firstWritePromise, pipePromise]))
    .then(() => {
      assert_array_equals(rs.events, ['pull', 'pull', 'pull']);
      assert_array_equals(ws.events, ['write', 'a', 'write', 'b','write', 'c','write', 'd', 'close']);
    });

}, 'Piping from a ReadableStream to a WritableStream that desires more chunks before finishing with previous ones');

promise_test(() => {

  const desiredSizes = [];
  const rs = recordingReadableStream({
    start(controller) {
      delay(100).then(() => enqueue('a'));
      delay(200).then(() => enqueue('b'));
      delay(300).then(() => enqueue('c'));
      delay(400).then(() => enqueue('d'));
      delay(500).then(() => controller.close());

      function enqueue(chunk) {
        controller.enqueue(chunk);
        desiredSizes.push(controller.desiredSize);
      }
    }
  });

  const chunksFinishedWriting = [];
  const writableStartPromise = Promise.resolve();
  const ws = recordingWritableStream({
    start() {
      return writableStartPromise;
    },
    write(chunk) {
      return delay(350).then(() => {
        chunksFinishedWriting.push(chunk);
      });
    }
  });

  return writableStartPromise.then(() => {
    return Promise.all([
      rs.pipeTo(ws).then(() => {
        assert_array_equals(desiredSizes, [1, 0, -1, -2], 'backpressure must have been exerted at the source');
        assert_array_equals(chunksFinishedWriting, ['a', 'b', 'c', 'd'], 'all chunks started writing');

        assert_array_equals(rs.eventsWithoutPulls, [], 'nothing unexpected should happen to the ReadableStream');
        assert_array_equals(ws.events, ['write', 'a', 'write', 'b', 'write', 'c', 'write', 'd', 'close'],
          'all chunks were written (and the WritableStream closed)');
      }),

      delay(125).then(() => {
        assert_array_equals(chunksFinishedWriting, [], 'at t = 125 ms, zero chunks must have finished writing');
        assert_array_equals(ws.events, ['write', 'a'], 'at t = 125 ms, one chunk must have been written');

        // When 'a' (the very first chunk) was enqueued, it was immediately used to fulfill the outstanding read request
        // promise, leaving the queue empty.
        assert_array_equals(desiredSizes, [1],
          'at t = 125 ms, the desiredSize at the last enqueue (100 ms) must have been 1');
        assert_equals(rs.controller.desiredSize, 1, 'at t = 125 ms, the current desiredSize must be 1');
      }),

      delay(225).then(() => {
        assert_array_equals(chunksFinishedWriting, [], 'at t = 225 ms, zero chunks must have finished writing');
        assert_array_equals(ws.events, ['write', 'a'], 'at t = 225 ms, one chunk must have been written');

        // When 'b' was enqueued at 200 ms, the queue was also empty, since immediately after enqueuing 'a' at
        // t = 100 ms, it was dequeued in order to fulfill the read() call that was made at time t = 0. Thus the queue
        // had size 1 (thus desiredSize of 0).
        assert_array_equals(desiredSizes, [1, 0],
          'at t = 225 ms, the desiredSize at the last enqueue (200 ms) must have been 0');
        assert_equals(rs.controller.desiredSize, 0, 'at t = 225 ms, the current desiredSize must be 0');
      }),

      delay(325).then(() => {
        assert_array_equals(chunksFinishedWriting, [], 'at t = 325 ms, zero chunks must have finished writing');
        assert_array_equals(ws.events, ['write', 'a'], 'at t = 325 ms, one chunk must have been written');

        // When 'c' was enqueued at 300 ms, the queue was not empty; it had 'b' in it, since 'b' will not be read until
        // the first write completes at 450 ms. Thus, the queue size is 2 after enqueuing 'c', giving a desiredSize of
        // -1.
        assert_array_equals(desiredSizes, [1, 0, -1],
          'at t = 325 ms, the desiredSize at the last enqueue (300 ms) must have been -1');
        assert_equals(rs.controller.desiredSize, -1, 'at t = 325 ms, the current desiredSize must be -1');
      }),

      delay(425).then(() => {
        assert_array_equals(chunksFinishedWriting, [], 'at t = 425 ms, zero chunks must have finished writing');
        assert_array_equals(ws.events, ['write', 'a'], 'at t = 425 ms, one chunk must have been written');

        // When 'd' was enqueued at 400 ms, the situation is the same as before, leading to a queue containing 'b', 'c',
        // and 'd'. (Remember the first write will only finish at 100 ms + 350 ms = 450 ms.)
        assert_array_equals(desiredSizes, [1, 0, -1, -2],
          'at t = 425 ms, the desiredSize at the last enqueue (400 ms) must have been -2');
        assert_equals(rs.controller.desiredSize, -2, 'at t = 425 ms, the current desiredSize must be -2');
      }),

      delay(475).then(() => {
        assert_array_equals(chunksFinishedWriting, ['a'], 'at t = 475 ms, one chunk must have finished writing');
        assert_array_equals(ws.events, ['write', 'a', 'write', 'b'],
          'at t = 475 ms, two chunks must have been written');

        assert_equals(rs.controller.desiredSize, -1, 'at t = 475 ms, the current desiredSize must be -1');
      })
    ]);
  });
}, 'Piping to a WritableStream that does not consume the writes fast enough exerts backpressure on the ReadableStream');

done();
