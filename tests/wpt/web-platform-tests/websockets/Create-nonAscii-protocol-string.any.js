// META: script=websocket.sub.js

test(function() {
  var nonAsciiProtocol = "\u0080echo";
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonAsciiProtocol(nonAsciiProtocol)
  });
}, "Create WebSocket - Pass a valid URL and a protocol string with non-ascii values - SYNTAX_ERR is thrown")
