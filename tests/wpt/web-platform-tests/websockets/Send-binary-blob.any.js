// META: script=constants.sub.js
// META: variant=
// META: variant=?wpt_flags=h2
// META: variant=?wss

var test = async_test("Send binary data on a WebSocket - Blob - Connection should be closed");

var data = "";
var datasize = 65000;
var isOpenCalled = false;
var isMessageCalled = false;

var wsocket = CreateWebSocket(false, false);

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.binaryType = "blob";
  for (var i = 0; i < datasize; i++)
    data += String.fromCharCode(0);
  data = new Blob([data]);
  isOpenCalled = true;
  wsocket.send(data);
}), true);

wsocket.addEventListener('message', test.step_func(function(evt) {
  isMessageCalled = true;
  assert_true(evt.data instanceof Blob);
  assert_equals(evt.data.size, datasize);
  wsocket.close();
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_true(isMessageCalled, "message should be received");
  assert_true(evt.wasClean, "wasClean should be true");
  test.done();
}), true);
