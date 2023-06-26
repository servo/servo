// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close should not emit until handshake completes - Connection should be closed");

var wsocket = new WebSocket(`${SCHEME_DOMAIN_PORT}/delayed-passive-close`);
var startTime;
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  startTime = performance.now();
  wsocket.close();
  isOpenCalled = true;
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  const elapsed = performance.now() - startTime;
  assert_true(isOpenCalled, "WebSocket connection should be open");
  assert_equals(wsocket.readyState, 3, "readyState should be 3(CLOSED)");
  assert_equals(evt.wasClean, true, "wasClean should be TRUE");
  const jitterAllowance = 100;
  assert_greater_than_equal(elapsed, 1000 - jitterAllowance,
    'one second should have elapsed')
  test.done();
}), true);
