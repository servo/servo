// META: script=websocket.sub.js

var test = async_test("Create Secure WebSocket - Close the Connection - close(1005) - see '7.1.5.  The WebSocket Connection Close Code' in http://www.ietf.org/rfc/rfc6455.txt");

var wsocket = CreateWebSocket(true, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  assert_throws_dom("INVALID_ACCESS_ERR", function() {
    wsocket.close(1005, "1005 - reserved code")
  });
  test.done();
}), true);
