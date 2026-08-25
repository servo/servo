// META: script=../../../constants.sub.js
// META: variant=?default
// META: variant=?wpt_flags=h2
// META: variant=?wss

async_test(t => {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/handshake_sleep_2');
  let closeMethodReturned = false;
  let errorEventSeen = false;
  let closeEventSeen = false;
  ws.onerror = t.step_func(() => {
    assert_true(closeMethodReturned, 'ws.close() should have returned');
    assert_false(errorEventSeen, 'error event should only fire once');
    errorEventSeen = true;
    assert_false(closeEventSeen, 'error event should come before close event');
  });
  ws.onclose = t.step_func_done(() => {
    assert_true(closeMethodReturned, 'ws.close() should have returned');
    assert_true(errorEventSeen, 'error event should have fired');
    assert_false(closeEventSeen, 'close event should only fire once');
    closeEventSeen = true;
    assert_equals(ws.readyState, WebSocket.CLOSED,
                  'readyState should be CLOSED');
  });
  assert_equals(ws.readyState, WebSocket.CONNECTING,
                'readyState should be CONNECTING');
  ws.close();
  closeMethodReturned = true;
  assert_equals(ws.readyState, WebSocket.CLOSING,
                'readyState should be CLOSING');
}, 'close event should be fired asynchronously when WebSocket is connecting');
