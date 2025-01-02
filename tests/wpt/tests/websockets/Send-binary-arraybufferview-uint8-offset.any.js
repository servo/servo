// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wpt_flags=h2
// META: variant=?wss

var test = async_test("Send binary data on a WebSocket - ArrayBufferView - Uint8Array with offset - Connection should be closed");

var data = "";
var datasize = 8;
var view;
var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;
var isMessageCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.binaryType = "arraybuffer";
  data = new ArrayBuffer(datasize);
  view = new Uint8Array(data, 2);
  for (var i = 0; i < 8; i++) {
    view[i] = i;
  }
  wsocket.send(view);
  isOpenCalled = true;
}), true);

wsocket.addEventListener('message', test.step_func(function(evt) {
  isMessageCalled = true;
  var resultView = new Uint8Array(evt.data);
  for (var i = 0; i < resultView.length; i++) {
    assert_equals(resultView[i], view[i], "ArrayBufferView returned is the same");
  }
  wsocket.close();
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_true(isMessageCalled, "message should be received")
  assert_equals(evt.wasClean, true, "wasClean should be true");
  test.done();
}), true);
