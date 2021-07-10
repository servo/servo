// META: script=websocket.sub.js

test(function() {
  var urlNonDefaultPort = "wss://" + __SERVER__NAME + ":" + __NEW__SECURE__PORT + "/" + __PATH;
  var wsocket = new WebSocket(urlNonDefaultPort);
  assert_equals(wsocket.url, urlNonDefaultPort, "wsocket.url is set correctly");
}, "Create Secure WebSocket - wsocket.url should be set correctly")
