// META: timeout=long
// META: script=websocket.sub.js

var testOpen = async_test("Create Secure WebSocket - wsocket.extensions should be set to '' after connection is established - Connection should be opened");
var testClose = async_test("Create Secure WebSocket - wsocket.extensions should be set to '' after connection is established - Connection should be closed");

var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func_done(function(evt) {
  wsocket.close();
  isOpenCalled = true;
  assert_equals(wsocket.extensions, "", "extensions should be empty");
}), true);

wsocket.addEventListener('close', testClose.step_func_done(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be closed");
  assert_equals(evt.wasClean, true, "wasClean should be true");
}), true);
