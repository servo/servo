// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close(2999, reason) - INVALID_ACCESS_ERR is thrown");

var wsocket = CreateWebSocket(false, false);

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws_dom("INVALID_ACCESS_ERR", function() {
    wsocket.close(2999, "Close not in range 3000-4999")
  });
  test.done();
}), true);

wsocket.addEventListener('close', test.unreached_func('close event should not fire'), true);
