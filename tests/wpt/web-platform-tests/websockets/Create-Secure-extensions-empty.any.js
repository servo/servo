// META: script=websocket.sub.js

var testOpen = async_test("W3C WebSocket API - Create Secure WebSocket - wsocket.extensions should be set to '' after connection is established - Connection should be opened");
var testClose = async_test("W3C WebSocket API - Create Secure WebSocket - wsocket.extensions should be set to '' after connection is established - Connection should be closed");

var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  assert_equals(wsocket.extensions, "", "extensions should be empty");
  wsocket.close();
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be closed");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
