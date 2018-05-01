// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonWsScheme()
  });
}, "W3C WebSocket API - Create WebSocket - Pass a URL with a non ws/wss scheme - SYNTAX_ERR is thrown")
