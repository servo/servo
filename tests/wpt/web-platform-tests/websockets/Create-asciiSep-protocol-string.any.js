// META: script=websocket.sub.js

test(function() {
  var asciiWithSep = "/echo";
  var wsocket;
  assert_throws("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithAsciiSep(asciiWithSep)
  });
}, "W3C WebSocket API - Create WebSocket - Pass a valid URL and a protocol string with an ascii separator character - SYNTAX_ERR is thrown")
