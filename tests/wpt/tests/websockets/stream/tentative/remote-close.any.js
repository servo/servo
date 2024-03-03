// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

'use strict';

promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?code=1000`);
  const { readable, writable } = await wss.opened;
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1000, 'code should be 1000');
  assert_equals(reason, '', 'reason should be empty');
  const { value, done } = await readable.getReader().read();
  assert_true(done, 'readable should be closed');
  await promise_rejects_dom(t, 'InvalidStateError', writable.getWriter().ready,
                            'writable should be errored');
}, 'clean close should be clean');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close`);
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1005, 'code should be No Status Rcvd');
  assert_equals(reason, '', 'reason should be empty');
}, 'close frame with no body should result in status code 1005');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?code=4000&reason=robot`);
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 4000, 'code should be 4000');
  assert_equals(reason, 'robot', 'reason should be set');
}, 'reason should be passed through');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?code=4000&` +
                                  'reason=%E3%83%AD%E3%83%9C%E3%83%83%E3%83%88');
  const { reason } = await wss.closed;
  assert_equals(reason, 'ロボット', 'reason should be set');
}, 'UTF-8 reason should work');

promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?code=4567`);
  const { writable } = await wss.opened;
  const veryLargeMessage = new Uint8Array(20 * 1024 * 1024);  // 20MB.
  const writePromise = writable.getWriter().write(veryLargeMessage);
  const closedError = await wss.closed.then(t.unreached_func('closed should reject'), e => e);
  assert_equals(closedError.constructor, WebSocketError, 'error should be WebSocketError');
  assert_equals(closedError.closeCode, 4567, 'closeCode should be set');
  promise_rejects_js(t, WebSocketError, writePromise, 'write() should reject');
}, 'close with unwritten data should not be considered clean');

promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?code=4222&reason=remote`);
  await wss.opened;
  wss.close({closeCode: 4111, reason: 'local'});
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 4222, 'remote code should be used');
  assert_equals(reason, 'remote', 'remote reason should be used');
}, 'remote code and reason should be used');

promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/remote-close?abrupt=1`);
  const { readable, writable } = await wss.opened;
  const closedError = await wss.closed.then(t.unreached_func('closed should reject'), e => e);
  assert_equals(closedError.constructor, WebSocketError, 'error should be a WebSocketError');
  assert_equals(closedError.name, 'WebSocketError', 'error name should be WebSocketError');
  assert_equals(closedError.closeCode, 1006, 'code should be Abnormal Closure');
  await promise_rejects_exactly(t, closedError, readable.getReader().read(),
                                'readable should be errored with the same object');
  await promise_rejects_exactly(t, closedError, writable.getWriter().ready,
                                'writable should be errored with the same object');
}, 'abrupt close should give an error');
