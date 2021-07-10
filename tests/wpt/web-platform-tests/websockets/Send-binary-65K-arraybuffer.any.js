// META: script=websocket.sub.js

var testOpen = async_test("Send 65K binary data on a WebSocket - ArrayBuffer - Connection should be opened");
var testMessage = async_test("Send 65K binary data on a WebSocket - ArrayBuffer - Message should be received");
var testClose = async_test("Send 65K binary data on a WebSocket - ArrayBuffer - Connection should be closed");

var data = "";
var datasize = 65000;
var wsocket = CreateWebSocket(false, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.binaryType = "arraybuffer";
  data = new ArrayBuffer(datasize);
  wsocket.send(data);
  assert_equals(datasize, wsocket.bufferedAmount);
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('message', testMessage.step_func(function(evt) {
  assert_equals(evt.data.byteLength, datasize);
  wsocket.close();
  testMessage.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
