// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(function() {
  var asciiWithSep = "/echo";
  var wsocket;
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket = CreateWebSocketWithAsciiSep(asciiWithSep)
  });
}, "Create WebSocket - Pass a valid URL and a protocol string with an ascii separator character - SYNTAX_ERR is thrown")
