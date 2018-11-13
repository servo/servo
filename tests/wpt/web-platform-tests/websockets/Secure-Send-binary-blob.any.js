// META: script=websocket.sub.js

var testOpen = async_test("W3C WebSocket API - Send binary data on a Secure WebSocket - Blob - Connection should be opened");
var testMessage = async_test("W3C WebSocket API - Send binary data on a Secure WebSocket - Blob - Message should be received");
var testClose = async_test("W3C WebSocket API - Send binary data on a Secure WebSocket - Blob - Connection should be closed");

var data = "";
var datasize = 65000;
var isOpenCalled = false;

var wsocket = CreateWebSocket(true, false, false);

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.binaryType = "blob";
  for (var i = 0; i < datasize; i++)
    data += String.fromCharCode(0);
  data = new Blob([data]);
  isOpenCalled = true;
  wsocket.send(data);
  testOpen.done();
}), true);

wsocket.addEventListener('message', testMessage.step_func(function(evt) {
  assert_true(evt.data instanceof Blob);
  assert_equals(evt.data.size, datasize);
  wsocket.close();
  testMessage.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_true(evt.wasClean, "wasClean should be true");
  testClose.done();
}), true);
