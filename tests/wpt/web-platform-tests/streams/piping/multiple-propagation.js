'use strict';

if (self.importScripts) {
  self.importScripts('/resources/testharness.js');
  self.importScripts('../resources/test-utils.js');
  self.importScripts('../resources/recording-streams.js');
}

const error1 = new Error('error1!');
error1.name = 'error1';

const error2 = new Error('error2!');
error2.name = 'error2';

promise_test(t => {
  const rs = recordingReadableStream({
    start(c) {
      c.error(error1);
    }
  });
  const ws = recordingWritableStream({
    start(c) {
      c.error(error2);
    }
  });

  // Trying to abort a stream that was errored will give that error back
  return promise_rejects(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the writable stream\'s error').then(() => {
    assert_array_equals(rs.events, []);
    assert_array_equals(ws.events, []);

    return Promise.all([
      promise_rejects(t, error1, rs.getReader().closed, 'the readable stream must be errored with error1'),
      promise_rejects(t, error2, ws.getWriter().closed, 'the writable stream must be errored with error2')
    ]);
  });

}, 'Piping from an errored readable stream to an errored writable stream');

promise_test(t => {
  const rs = recordingReadableStream({
    start(c) {
      c.error(error1);
    }
  });
  const ws = recordingWritableStream({
    start(c) {
      c.error(error2);
    }
  });

  return promise_rejects(t, error1, rs.pipeTo(ws, { preventAbort: true }),
    'pipeTo must reject with the readable stream\'s error')
  .then(() => {
    assert_array_equals(rs.events, []);
    assert_array_equals(ws.events, []);

    return Promise.all([
      promise_rejects(t, error1, rs.getReader().closed, 'the readable stream must be errored with error1'),
      promise_rejects(t, error2, ws.getWriter().closed, 'the writable stream must be errored with error2')
    ]);
  });

}, 'Piping from an errored readable stream to an errored writable stream; preventAbort = true');

promise_test(t => {
  const rs = recordingReadableStream({
    start(c) {
      c.error(error1);
    }
  });
  const ws = recordingWritableStream();
  const writer = ws.getWriter();
  const closePromise = writer.close();
  writer.releaseLock();

  return promise_rejects(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the readable stream\'s error').then(() => {
    assert_array_equals(rs.events, []);
    assert_array_equals(ws.events, ['abort', error1]);

    return Promise.all([
      promise_rejects(t, error1, rs.getReader().closed, 'the readable stream must be errored with error1'),
      promise_rejects(t, new TypeError(), ws.getWriter().closed,
        'closed must reject with a TypeError indicating the writable stream was aborted'),
      promise_rejects(t, new TypeError(), closePromise,
        'close() must reject with a TypeError indicating the writable stream was aborted'),
    ]);
  });

}, 'Piping from an errored readable stream to a closing writable stream');

promise_test(t => {
  const rs = recordingReadableStream({
    start(c) {
      c.error(error1);
    }
  });
  const ws = recordingWritableStream();
  const writer = ws.getWriter();
  const closePromise = writer.close();
  writer.releaseLock();

  return flushAsyncEvents().then(() => {
    return promise_rejects(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the readable stream\'s error').then(() => {
      assert_array_equals(rs.events, []);
      assert_array_equals(ws.events, ['close']);

      return Promise.all([
        promise_rejects(t, error1, rs.getReader().closed, 'the readable stream must be errored with error1'),
        ws.getWriter().closed,
        closePromise
      ]);
    });
  });

}, 'Piping from an errored readable stream to a closed writable stream');

promise_test(t => {
  const rs = recordingReadableStream({
    start(c) {
      c.close();
    }
  });
  const ws = recordingWritableStream({
    start(c) {
      c.error(error1);
    }
  });

  return promise_rejects(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the writable stream\'s error').then(() => {
    assert_array_equals(rs.events, []);
    assert_array_equals(ws.events, []);

    return Promise.all([
      rs.getReader().closed,
      promise_rejects(t, error1, ws.getWriter().closed, 'the writable stream must be errored with error1')
    ]);
  });

}, 'Piping from a closed readable stream to an errored writable stream');

promise_test(() => {
  const rs = recordingReadableStream({
    start(c) {
      c.close();
    }
  });
  const ws = recordingWritableStream();
  const writer = ws.getWriter();
  writer.close();
  writer.releaseLock();

  return rs.pipeTo(ws).then(() => {
    assert_array_equals(rs.events, []);
    assert_array_equals(ws.events, ['close']);

    return Promise.all([
      rs.getReader().closed,
      ws.getWriter().closed
    ]);
  });

}, 'Piping from a closed readable stream to a closed writable stream');

done();
