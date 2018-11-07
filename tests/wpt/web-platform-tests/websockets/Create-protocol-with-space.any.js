// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithSpaceInProtocol("ec ho")
  });
}, "Create WebSocket - Pass a valid URL and a protocol string with a space in it - SYNTAX_ERR is thrown")
