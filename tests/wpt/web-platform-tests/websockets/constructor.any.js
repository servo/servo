// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

test(function() {
  var ws = new WebSocket(SCHEME_DOMAIN_PORT + "/" + __PATH,
    "echo", "Stray argument")
  assert_true(ws instanceof WebSocket, "Expected a WebSocket instance.")
}, "Calling the WebSocket constructor with too many arguments should not throw.")
