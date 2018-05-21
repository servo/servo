// META: script=websocket.sub.js

var test = async_test("W3C WebSocket API - Create Secure WebSocket - Close the Connection - close(2999, reason) - INVALID_ACCESS_ERR is thrown");

var wsocket = CreateWebSocket(true, false, false);

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws("INVALID_ACCESS_ERR", function() {
    wsocket.close(2999, "Close not in range 3000-4999")
  });
  test.done();
}), true);
