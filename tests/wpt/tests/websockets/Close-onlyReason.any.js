// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close(only reason) - INVALID_ACCESS_ERR is thrown");

var wsocket = CreateWebSocket(false, false);

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws_dom("INVALID_ACCESS_ERR", function() {
    wsocket.close("Close with only reason")
  });
  test.done();
}), true);

wsocket.addEventListener('close', test.unreached_func('close event should not fire'), true);
