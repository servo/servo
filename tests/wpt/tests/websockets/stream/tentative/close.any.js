// META: script=../../constants.sub.js
// META: script=resources/url-constants.js
// META: global=window,worker
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

'use strict';

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  wss.close({ closeCode: 3456, reason: 'pizza' });
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 3456, 'code should match');
  assert_equals(reason, 'pizza', 'reason should match');
}, 'close code should be sent to server and reflected back');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  wss.close();
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'no close argument should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  wss.close({});
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'unspecified close code should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  wss.close({reason: ''});
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'unspecified close code with empty reason should send empty Close frame');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  wss.close({reason: 'non-empty'});
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1000, 'code should be set');
  assert_equals(reason, 'non-empty', 'reason should match');
}, 'unspecified close code with non-empty reason should set code to 1000');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  assert_throws_js(TypeError, () => wss.close(true),
                   'close should throw a TypeError');
}, 'close(true) should throw a TypeError');

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  await wss.opened;
  const reason = '.'.repeat(124);
  assert_throws_dom('SyntaxError', () => wss.close({ reason }),
                    'close should throw a SyntaxError');
}, 'close() with an overlong reason should throw');

function IsWebSocketError(e) {
  return e.constructor == WebSocketError;
}

promise_test(t => {
  const wss = new WebSocketStream(ECHOURL);
  wss.close();
  return Promise.all([
    wss.opened.then(t.unreached_func('should have rejected')).catch(e => assert_true(IsWebSocketError(e))),
    wss.closed.then(t.unreached_func('should have rejected')).catch(e => assert_true(IsWebSocketError(e))),
  ]);
}, 'close during handshake should work');

for (const invalidCode of [999, 1001, 2999, 5000]) {
  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    await wss.opened;
    assert_throws_dom('InvalidAccessError', () => wss.close({ closeCode: invalidCode }),
                      'close should throw an InvalidAccessError');
  }, `close() with invalid code ${invalidCode} should throw`);
}

promise_test(async () => {
  const wss = new WebSocketStream(ECHOURL);
  const { writable } = await wss.opened;
  writable.getWriter().close();
  const { closeCode, reason } = await wss.closed;
  assert_equals(closeCode, 1005, 'code should be unset');
  assert_equals(reason, '', 'reason should be empty');
}, 'closing the writable should result in a clean close');

promise_test(async () => {
  const wss = new WebSocketStream(`${BASEURL}/delayed-passive-close`);
  const { writable } = await wss.opened;
  const startTime = performance.now();
  await writable.getWriter().close();
  const elapsed = performance.now() - startTime;
  const jitterAllowance = 100;
  assert_greater_than_equal(elapsed, 1000 - jitterAllowance,
                            'one second should have elapsed');
}, 'writer close() promise should not resolve until handshake completes');

promise_test(async t => {
  const wss = new WebSocketStream(`${BASEURL}/passive-close-abort`);
  await wss.opened;
  wss.close({closeCode: 4000, reason: 'because'});
  const error = await wss.closed.then(t.unreached_func('closed should reject'), e => e);
  assert_equals(error.constructor, WebSocketError, 'error should be WebSocketError');
  assert_equals(error.closeCode, 1006, 'close code should be Abnormal Closure');
}, 'incomplete closing handshake should be considered unclean close');

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
    const info = await wss.opened;
    info[stream][method]();
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} should result in a clean close`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.opened;
    info[stream][method]({ closeCode: 3333, reason: 'obsolete' });
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with attributes not wrapped in a WebSocketError should be ignored`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.opened;
    info[stream][method](new WebSocketError('', { closeCode: 3333 }));
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 3333, 'code should be used');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with a code should send that code`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.opened;
    info[stream][method](new WebSocketError('', { closeCode: 3456, reason: 'set' }));
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 3456, 'code should be used');
    assert_equals(reason, 'set', 'reason should be used');
  }, `${voweling} the ${stream} with a code and reason should use them`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.opened;
    info[stream][method](new WebSocketError('', { reason: 'specified' }));
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 1000, 'code should be defaulted');
    assert_equals(reason, 'specified', 'reason should be used');
  }, `${voweling} the ${stream} with a reason but no code should default the close code`);

  promise_test(async () => {
    const wss = new WebSocketStream(ECHOURL);
    const info = await wss.opened;
    const domException = new DOMException('yes', 'DataCloneError');
    domException.closeCode = 1000;
    domException.reason = 'should be ignored';
    info[stream][method](domException);
    const { closeCode, reason } = await wss.closed;
    assert_equals(closeCode, 1005, 'code should be unset');
    assert_equals(reason, '', 'reason should be empty');
  }, `${voweling} the ${stream} with a DOMException not set code or reason`);

}
