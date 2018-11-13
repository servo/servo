// META: script=websocket.sub.js

var testOpen = async_test("W3C WebSocket API - Send  data on a Secure WebSocket - Connection should be opened");
var testMessage = async_test("W3C WebSocket API - Send data on a Secure WebSocket - Message should be received");
var testClose = async_test("W3C WebSocket API - Send data on a Secure WebSocket - Connection should be closed");

var data = "Message to send";
var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.send(data);
  assert_equals(data.length, wsocket.bufferedAmount);
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('message', testMessage.step_func(function(evt) {
  assert_equals(evt.data, data);
  wsocket.close();
  testMessage.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
