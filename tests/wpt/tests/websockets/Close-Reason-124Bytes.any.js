// META: script=constants.sub.js
// META: variant=?default
// META: variant=?wpt_flags=h2
// META: variant=?wss

var test = async_test("Create WebSocket - Close the Connection - close(code, 'reason more than 123 bytes') - SYNTAX_ERR is thrown");

var wsocket = CreateWebSocket(false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  var reason = "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123";
  assert_equals(reason.length, 124);
  assert_throws_dom("SYNTAX_ERR", function() {
    wsocket.close(1000, reason)
  });
  test.done();
}), true);

wsocket.addEventListener('close', test.unreached_func('close event should not fire'), true);
