// META: script=websocket.sub.js

var testOpen = async_test("W3C WebSocket API - Send binary data on a WebSocket - ArrayBufferView - Uint8Array with offset and length - Connection should be opened");
var testMessage = async_test("W3C WebSocket API - Send binary data on a WebSocket - ArrayBufferView - Uint8Array with offset and length - Message should be received");
var testClose = async_test("W3C WebSocket API - Send binary data on a WebSocket - ArrayBufferView - Uint8Array with offset and length - Connection should be closed");

var data = "";
var datasize = 8;
var view;
var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.binaryType = "arraybuffer";
  data = new ArrayBuffer(datasize);
  view = new Uint8Array(data, 2, 4);
  for (var i = 0; i < 8; i++) {
    view[i] = i;
  }
  wsocket.send(view);
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('message', testMessage.step_func(function(evt) {
  var resultView = new Uint8Array(evt.data);
  for (var i = 0; i < resultView.length; i++) {
    assert_equals(resultView[i], view[i], "ArrayBufferView returned is the same");
  }
  wsocket.close();
  testMessage.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
