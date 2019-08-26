// META: script=../websocket.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  wss.close({code: 3456, reason: 'pizza'});
  const { code, reason } = await wss.closed;
  assert_equals(code, 3456, 'code should match');
  assert_equals(reason, 'pizza', 'reason should match');
}, 'close code should be sent to server and reflected back');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  wss.close();
  const { code, reason } = await wss.closed;
  assert_equals(code, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'no close argument should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  wss.close({});
  const { code, reason } = await wss.closed;
  assert_equals(code, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'unspecified close code should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  wss.close({reason: ''});
  const { code, reason } = await wss.closed;
  assert_equals(code, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'unspecified close code with empty reason should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  wss.close({reason: 'non-empty'});
  const { code, reason } = await wss.closed;
  assert_equals(code, 1000, 'code should be set');
  assert_equals(reason, 'non-empty', 'reason should match');
}, 'unspecified close code with non-empty reason should set code to 1000');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  assert_throws(new TypeError(), () => wss.close(true),
                'close should throw a TypeError');
}, 'close(true) should throw a TypeError');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.connection;
  const reason = '.'.repeat(124);
  assert_throws('SyntaxError', () => wss.close({ reason }),
                'close should throw a TypeError');
}, 'close() with an overlong reason should throw');

promise_test(t => {
  const wss = new WebSocketStream(ECHOURL);
  wss.close();
  return Promise.all([
    promise_rejects(t, 'NetworkError', wss.connection,
                    'connection promise should reject'),
    promise_rejects(t, 'NetworkError', wss.closed,
                    'closed promise should reject')]);
}, 'close during handshake should work');

for (const invalidCode of [999, 1001, 2999, 5000]) {
  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    await wss.connection;
    assert_throws('InvalidAccessError', () => wss.close({ code: invalidCode }),
                  'close should throw a TypeError');
  }, `close() with invalid code ${invalidCode} should throw`);
}

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  const { writable } = await wss.connection;
  writable.getWriter().close();
  const { code, reason } = await wss.closed;
  assert_equals(code, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'closing the writable should result in a clean close');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/delayed-passive-close`);
  const { writable } = await wss.connection;
  const startTime = performance.now();
  await writable.getWriter().close();
  const elapsed = performance.now() - startTime;
  const jitterAllowance = 100;
  assert_greater_than_equal(elapsed, 1000 - jitterAllowance,
                            'one second should have elapsed');
}, 'writer close() promise should not resolve until handshake completes');

const abortOrCancel = [
  {
    method: 'abort',
    voweling: 'aborting',
    stream: 'writable',
  },
  {
    method: 'cancel',
    voweling: 'canceling',
    stream: 'readable',
  },
];

for (const { method, voweling, stream } of abortOrCancel) {

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]();
    const { code, reason } = await wss.closed;
    assert_equals(code, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} should result in a clean close`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]({ code: 3333 });
    const { code, reason } = await wss.closed;
    assert_equals(code, 3333, 'code should be used');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with a code should send that code`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]({ code: 3456, reason: 'set' });
    const { code, reason } = await wss.closed;
    assert_equals(code, 3456, 'code should be used');
    assert_equals(reason, 'set', 'reason should be used');
  }, `${voweling} the ${stream} with a code and reason should use them`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]({ reason: 'specified' });
    const { code, reason } = await wss.closed;
    assert_equals(code, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with a reason but no code should be ignored`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]({ code: 999 });
    const { code, reason } = await wss.closed;
    assert_equals(code, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with an invalid code should be ignored`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method]({ code: 1000, reason: 'x'.repeat(128) });
    const { code, reason } = await wss.closed;
    assert_equals(code, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with an invalid reason should be ignored`);

  // DOMExceptions are only ignored because the |code| attribute is too small to
  // be a valid WebSocket close code.
  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.connection;
    info[stream][method](new DOMException('yes', 'DataCloneError'));
    const { code, reason } = await wss.closed;
    assert_equals(code, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with a DOMException should be ignored`);

}
