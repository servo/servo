// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithRepeatedProtocols()
  });
}, "W3C WebSocket API - Create WebSocket - Pass a valid URL and an array of protocol strings with repeated values - SYNTAX_ERR is thrown")
