// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(function() {
  var nonAsciiProtocol = "\u0080echo";
  var wsocket;
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketNonAsciiProtocol(nonAsciiProtocol)
  });
}, "Create WebSocket - Pass a valid URL and a protocol string with non-ascii values - SYNTAX_ERR is thrown")
