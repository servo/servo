// META: script=websocket.sub.js

var test = async_test("Create Secure WebSocket - Close the Connection - close(only reason) - INVALID_ACCESS_ERR is thrown");

var wsocket = CreateWebSocket(true, false, false);

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws_dom("INVALID_ACCESS_ERR", function() {
    wsocket.close("Close with only reason")
  });
  test.done();
}), true);
