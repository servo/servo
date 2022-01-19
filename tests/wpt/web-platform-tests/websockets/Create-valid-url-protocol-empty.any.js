// META: script=constants.sub.js
// META: variant=
// META: variant=?wpt_flags=h2
// META: variant=?wss

test(function() {
  var wsocket = CreateWebSocket(true, false);
  assert_equals(wsocket.protocol, "", "protocol should be empty");
  wsocket.close();
}, "Create WebSocket - wsocket.protocol should be empty before connection is established")
