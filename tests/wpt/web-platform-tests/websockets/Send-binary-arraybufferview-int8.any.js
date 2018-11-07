// META: script=websocket.sub.js

var testOpen = async_test("Send binary data on a WebSocket - ArrayBufferView - Int8Array - Connection should be opened");
var testMessage = async_test("Send binary data on a WebSocket - ArrayBufferView - Int8Array - Message should be received");
var testClose = async_test("Send binary data on a WebSocket - ArrayBufferView - Int8Array - Connection should be closed");

var data = "";
var datasize = 8;
var int8View;
var wsocket = CreateWebSocket(false, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', testOpen.step_func(function(evt) {
  wsocket.binaryType = "arraybuffer";
  data = new ArrayBuffer(datasize);
  int8View = new Int8Array(data);
  for (var i = 0; i < 8; i++) {
    int8View[i] = i;
  }
  wsocket.send(int8View);
  isOpenCalled = true;
  testOpen.done();
}), true);

wsocket.addEventListener('message', testMessage.step_func(function(evt) {
  var resultView = new Int8Array(evt.data);
  for (var i = 0; i < resultView.length; i++) {
    assert_equals(resultView[i], int8View[i], "ArrayBufferView returned is the same");
  }
  wsocket.close();
  testMessage.done();
}), true);

wsocket.addEventListener('close', testClose.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(evt.wasClean, true, "wasClean should be true");
  testClose.done();
}), true);
