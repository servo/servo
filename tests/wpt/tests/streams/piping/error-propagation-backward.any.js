// META: global=window,worker,shadowrealm
// META: script=../resources/test-utils.js
// META: script=../resources/recording-streams.js
'use strict';

const error1 = new Error('error1!');
error1.name = 'error1';

const error2 = new Error('error2!');
error2.name = 'error2';


promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('Hello');
    },
    cancel() {
      throw error2;
    }
  });

  const ws = recordingWritableStream({
    write() {
      throw error1;
    }
  });

  return promise_rejects_exactly(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the cancel error').then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, ['write', 'Hello']);
  });

}, 'Errors must be propagated backward: becomes errored during piping due to write; preventCancel omitted; rejected ' +
   'cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('Hello');
    }
  });

  const ws = recordingWritableStream({
    write() {
      throw error1;
    }
  });

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true }), 'pipeTo must reject with the same error')
  .then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, ['write', 'Hello']);
  });

}, 'Errors must be propagated backward: becomes errored during piping due to write; preventCancel = true');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.enqueue('c');
    }
  });

  const ws = recordingWritableStream({
    write() {
      if (ws.events.length > 2) {
        return delay(0).then(() => {
          throw error1;
        });
      }
      return undefined;
    }
  });

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the same error').then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, ['write', 'a', 'write', 'b']);
  });

}, 'Errors must be propagated backward: becomes errored during piping due to write, but async; preventCancel = ' +
   'false; fulfilled cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.enqueue('c');
    },
    cancel() {
      throw error2;
    }
  });

  const ws = recordingWritableStream({
    write() {
      if (ws.events.length > 2) {
        return delay(0).then(() => {
          throw error1;
        });
      }
      return undefined;
    }
  });

  return promise_rejects_exactly(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the cancel error').then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, ['write', 'a', 'write', 'b']);
  });

}, 'Errors must be propagated backward: becomes errored during piping due to write, but async; preventCancel = ' +
   'false; rejected cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.enqueue('c');
    }
  });

  const ws = recordingWritableStream({
    write() {
      if (ws.events.length > 2) {
        return delay(0).then(() => {
          throw error1;
        });
      }
      return undefined;
    }
  });

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true }), 'pipeTo must reject with the same error')
  .then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, ['write', 'a', 'write', 'b']);
  });

}, 'Errors must be propagated backward: becomes errored during piping due to write, but async; preventCancel = true');

promise_test(t => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream();

  const pipePromise = promise_rejects_exactly(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the same error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; preventCancel omitted; fulfilled cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    cancel() {
      throw error2;
    }
  });

  const ws = recordingWritableStream();

  const pipePromise = promise_rejects_exactly(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the cancel error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; preventCancel omitted; rejected cancel promise');

promise_test(t => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream();

  const pipePromise = promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true }),
                                              'pipeTo must reject with the same error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; preventCancel = true');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.enqueue('c');
      controller.close();
    }
  });

  const ws = recordingWritableStream({
    write(chunk) {
      if (chunk === 'c') {
        return Promise.reject(error1);
      }
      return undefined;
    }
  });

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the same error').then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, ['write', 'a', 'write', 'b', 'write', 'c']);
  });

}, 'Errors must be propagated backward: becomes errored after piping due to last write; source is closed; ' +
   'preventCancel omitted (but cancel is never called)');

promise_test(t => {

  const rs = recordingReadableStream({
    start(controller) {
      controller.enqueue('a');
      controller.enqueue('b');
      controller.enqueue('c');
      controller.close();
    }
  });

  const ws = recordingWritableStream({
    write(chunk) {
      if (chunk === 'c') {
        return Promise.reject(error1);
      }
      return undefined;
    }
  });

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true }), 'pipeTo must reject with the same error')
    .then(() => {
      assert_array_equals(rs.eventsWithoutPulls, []);
      assert_array_equals(ws.events, ['write', 'a', 'write', 'b', 'write', 'c']);
    });

}, 'Errors must be propagated backward: becomes errored after piping due to last write; source is closed; ' +
   'preventCancel = true');

promise_test(t => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream(undefined, new CountQueuingStrategy({ highWaterMark: 0 }));

  const pipePromise = promise_rejects_exactly(t, error1, rs.pipeTo(ws), 'pipeTo must reject with the same error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; dest never desires chunks; preventCancel = ' +
   'false; fulfilled cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    cancel() {
      throw error2;
    }
  });

  const ws = recordingWritableStream(undefined, new CountQueuingStrategy({ highWaterMark: 0 }));

  const pipePromise = promise_rejects_exactly(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the cancel error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; dest never desires chunks; preventCancel = ' +
   'false; rejected cancel promise');

promise_test(t => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream(undefined, new CountQueuingStrategy({ highWaterMark: 0 }));

  const pipePromise = promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true }),
                                              'pipeTo must reject with the same error');

  t.step_timeout(() => ws.controller.error(error1), 10);

  return pipePromise.then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, []);
  });

}, 'Errors must be propagated backward: becomes errored after piping; dest never desires chunks; preventCancel = ' +
   'true');

promise_test(() => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream();

  ws.abort(error1);

  return rs.pipeTo(ws).then(
    () => assert_unreached('the promise must not fulfill'),
    err => {
      assert_equals(err, error1, 'the promise must reject with error1');

      assert_array_equals(rs.eventsWithoutPulls, ['cancel', err]);
      assert_array_equals(ws.events, ['abort', error1]);
    }
  );

}, 'Errors must be propagated backward: becomes errored before piping via abort; preventCancel omitted; fulfilled ' +
   'cancel promise');

promise_test(t => {

  const rs = recordingReadableStream({
    cancel() {
      throw error2;
    }
  });

  const ws = recordingWritableStream();

  ws.abort(error1);

  return promise_rejects_exactly(t, error2, rs.pipeTo(ws), 'pipeTo must reject with the cancel error')
    .then(() => {
      return ws.getWriter().closed.then(
        () => assert_unreached('the promise must not fulfill'),
        err => {
          assert_equals(err, error1, 'the promise must reject with error1');

          assert_array_equals(rs.eventsWithoutPulls, ['cancel', err]);
          assert_array_equals(ws.events, ['abort', error1]);
        }
      );
    });

}, 'Errors must be propagated backward: becomes errored before piping via abort; preventCancel omitted; rejected ' +
   'cancel promise');

promise_test(t => {

  const rs = recordingReadableStream();

  const ws = recordingWritableStream();

  ws.abort(error1);

  return promise_rejects_exactly(t, error1, rs.pipeTo(ws, { preventCancel: true })).then(() => {
    assert_array_equals(rs.eventsWithoutPulls, []);
    assert_array_equals(ws.events, ['abort', error1]);
  });

}, 'Errors must be propagated backward: becomes errored before piping via abort; preventCancel = true');

promise_test(t => {

  const rs = recordingReadableStream();

  let resolveWriteCalled;
  const writeCalledPromise = new Promise(resolve => {
    resolveWriteCalled = resolve;
  });

  const ws = recordingWritableStream({
    write() {
      resolveWriteCalled();
      return flushAsyncEvents();
    }
  });

  const pipePromise = rs.pipeTo(ws);

  rs.controller.enqueue('a');

  return writeCalledPromise.then(() => {
    ws.controller.error(error1);

    return promise_rejects_exactly(t, error1, pipePromise);
  }).then(() => {
    assert_array_equals(rs.eventsWithoutPulls, ['cancel', error1]);
    assert_array_equals(ws.events, ['write', 'a']);
  });

}, 'Errors must be propagated backward: erroring via the controller errors once pending write completes');
