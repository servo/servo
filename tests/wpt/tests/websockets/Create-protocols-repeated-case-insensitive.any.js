// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(function() {
  var wsocket;
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithRepeatedProtocolsCaseInsensitive()
  });
}, "Create WebSocket - Pass a valid URL and an array of protocol strings with repeated values but different case - SYNTAX_ERR is thrown")
