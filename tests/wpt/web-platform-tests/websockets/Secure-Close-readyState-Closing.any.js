// META: script=websocket.sub.js

var test = async_test("Create Secure WebSocket - Close the Connection - readyState should be in CLOSING state just before onclose is called");

var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.close();
  assert_equals(wsocket.readyState, 2, "readyState should be 2(CLOSING)");
  test.done();
}), true);
