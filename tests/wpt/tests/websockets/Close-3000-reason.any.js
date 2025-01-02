// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close(3000, reason) - readyState should be in CLOSED state and wasClean is TRUE - Connection should be closed");

var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.close(3000, "Clean Close with code - 3000");
  isOpenCalled = true;
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(wsocket.readyState, 3, "readyState should be 3(CLOSED)");
  assert_equals(evt.wasClean, true, "wasClean should be TRUE");
  test.done();
}), true);
