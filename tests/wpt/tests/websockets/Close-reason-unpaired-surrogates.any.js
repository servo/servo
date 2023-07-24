// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close(reason with unpaired surrogates) - connection should get closed");

var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;
var replacementChar = "\uFFFD";
var reason = "\uD807";

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.close(1000, reason);
  isOpenCalled = true;
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  assert_true(isOpenCalled, "WebSocket connection should be opened");
  assert_equals(evt.reason, replacementChar, "reason replaced with replacement character");
  test.done();
}), true);
