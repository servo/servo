// META: global=window,worker
// META: script=constants.sub.js

async_test(t => {
  const ws = CreateInsecureWebSocket();
  ws.onopen = t.unreached_func('open should not fire');
  ws.onerror = t.step_func(() => {
    assert_equals(ws.readyState, WebSocket.CLOSED);
  });
  ws.onclose = t.step_func_done(e => {
    assert_false(e.wasClean);
  });
}, 'opening an insecure WebSocket in a secure context should fail');
