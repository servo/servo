// META: script=websocket.sub.js

test(function() {
  var ws = new WebSocket("ws://" + __SERVER__NAME + ":" + __PORT + "/" + __PATH,
    "echo", "Stray argument")
  assert_true(ws instanceof WebSocket, "Expected a WebSocket instance.")
}, "Calling the WebSocket constructor with too many arguments should not throw.")
