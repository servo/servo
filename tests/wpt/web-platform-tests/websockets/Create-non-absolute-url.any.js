// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonAbsolute()
  });
}, "Create WebSocket - Pass a non absolute URL - SYNTAX_ERR is thrown")
