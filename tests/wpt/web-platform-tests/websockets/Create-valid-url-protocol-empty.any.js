// META: script=websocket.sub.js

test(function() {
  var wsocket = CreateWebSocket(false, true, false);
  assert_equals(wsocket.protocol, "", "protocol should be empty");
  wsocket.close();
}, "W3C WebSocket API - Create WebSocket - wsocket.protocol should be empty before connection is established")
