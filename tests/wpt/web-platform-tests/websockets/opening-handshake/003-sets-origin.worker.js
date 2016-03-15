importScripts("/resources/testharness.js");
importScripts('../constants.js?pipe=sub');
importScripts('../websocket.sub.js');

async_test(function(t) {
  var ws = new WebSocket(SCHEME_DOMAIN_PORT+'/origin');
  ws.onmessage = t.step_func(function(e) {
    assert_equals(e.data, location.protocol+'//'+location.host);
    ws.onclose = t.step_func(function(e) {
      assert_equals(e.wasClean, true);
      ws.onclose = t.step_func(function() {assert_unreached()});
      setTimeout(t.step_func_done(), 50)
    })
    ws.close();
  })
  ws.onerror = ws.onclose = t.step_func(function() {assert_unreached()});
}, "W3C WebSocket API - origin set in a Worker");

done();
