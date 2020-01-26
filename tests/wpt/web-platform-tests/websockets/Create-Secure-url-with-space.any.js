// META: script=websocket.sub.js

test(function() {
  var wsocket;
  var spaceUrl = "web platform.test";
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithSpaceInUrl(spaceUrl)
  });
}, "Create Secure WebSocket - Pass a URL with a space - SYNTAX_ERR should be thrown")
