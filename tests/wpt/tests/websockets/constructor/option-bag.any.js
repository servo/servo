// META: title=WebSockets: option bag constructor argument
// META: script=../constants.sub.js
// META: variant=?default
// META: variant=?wss

async_test(function(t) {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/echo', {});
  ws.onopen = t.step_func(function(e) {
    ws.close();
    t.done();
  });
  ws.onerror = t.unreached_func("error event should not have fired");
}, "Empty option bag should be accepted");

async_test(function(t) {
  const ws = new WebSocket(SCHEME_DOMAIN_PORT + '/protocol_array', { protocols: ['foobar', 'foobar2'] });
  ws.onmessage = t.step_func(function(e) {
    assert_equals(ws.protocol, 'foobar');
    assert_equals(e.data, 'foobar', 'message content should be "foobar"');
    ws.onclose = t.step_func(function(e) {
      t.done();
    });
    ws.close();
  });
  ws.onerror = t.unreached_func("error event should not have fired");
}, "Option bag with protocols array should be accepted");
