// META: script=websocket.sub.js

test(function() {
  var wsocket = CreateWebSocket(false, false, false);
  assert_throws("INVALID_STATE_ERR", function() {
    wsocket.send("Message to send")
  });
}, "W3C WebSocket API - Send data on a WebSocket before connection is opened - INVALID_STATE_ERR is returned")
