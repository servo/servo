// META: script=websocket.sub.js

var testOpen = async_test("Create Secure WebSocket - Close the Connection - close(reason with unpaired surrogates) - connection should get opened");
var testClose = async_test("Create Secure WebSocket - Close the Connection - close(reason with unpaired surrogates) - connection should get closed");

var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;
var replacementChar = "\uFFFD";
var reason = "\uD807";

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.close(1000, reason);
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be opened");
  assert_equals(evt.reason, replacementChar, "reason replaced with replacement character");
  testClose.done();
}), true);
