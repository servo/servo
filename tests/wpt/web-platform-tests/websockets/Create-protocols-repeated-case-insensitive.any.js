// META: script=websocket.sub.js

test(function() {
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithRepeatedProtocolsCaseInsensitive()
  });
}, "Create WebSocket - Pass a valid URL and an array of protocol strings with repeated values but different case - SYNTAX_ERR is thrown")
