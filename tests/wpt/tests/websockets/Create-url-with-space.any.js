// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wpt_flags=h2
// META: variant=?wss

test(function() {
  var wsocket;
  var spaceUrl = "web platform.test";
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithSpaceInUrl(spaceUrl)
  });
}, "Create WebSocket - Pass a URL with a space - SYNTAX_ERR should be thrown")
