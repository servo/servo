// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(function() {
  var wsocket;
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonWsScheme()
  });
}, "Create WebSocket - Pass a URL with a non ws/wss scheme - SYNTAX_ERR is thrown")
