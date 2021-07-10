// META: script=websocket.sub.js

var testOpen = async_test("Create Secure WebSocket - Pass a valid URL and protocol string - protocol should be set correctly - Connection should be opened");
var testClose = async_test("Create Secure WebSocket - Pass a valid URL and protocol string - Connection should be closed");

var wsocket = CreateWebSocket(true, true, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  assert_equals(wsocket.protocol, "echo", "protocol should be set to echo");
  wsocket.close();
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
