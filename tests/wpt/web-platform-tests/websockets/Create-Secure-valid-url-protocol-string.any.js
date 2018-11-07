// META: script=websocket.sub.js

var testOpen = async_test("Create Secure WebSocket - Check readyState is 1");
var testClose = async_test("Create Secure WebSocket - Pass a valid URL and protocol string - Connection should be closed");

var wsocket = CreateWebSocket(true, true, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  assert_equals(wsocket.readyState, 1, "readyState should be 1(OPEN)");
  wsocket.close();
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
