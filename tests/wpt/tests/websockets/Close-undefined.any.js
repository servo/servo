// META: script=constants.sub.js
// META: variant=
// META: variant=?wpt_flags=h2
// META: variant=?wss

var test = async_test();

var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  isOpenCalled = true;
  wsocket.close(undefined);
}), true);

wsocket.addEventListener('close', test.step_func(function(evt) {
  assert_true(isOpenCalled, 'open event must fire');
  test.done();
}), true);
