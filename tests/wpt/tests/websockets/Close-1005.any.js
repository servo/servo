// META: script=constants.sub.js
// META: variant=
// META: variant=?wss
// META: variant=?wpt_flags=h2

var test = async_test("Create WebSocket - Close the Connection - close(1005) - see '7.1.5.  The WebSocket Connection Close Code' in http://www.ietf.org/rfc/rfc6455.txt");

var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws_dom("INVALID_ACCESS_ERR", function() {
    wsocket.close(1005, "1005 - reserved code")
  });
  test.done();
}), true);

wsocket.addEventListener('close', test.unreached_func('close event should not fire'), true);
