// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonAbsolute()
  });
}, "W3C WebSocket API - Create WebSocket - Pass a non absolute URL - SYNTAX_ERR is thrown")
