// META: script=websocket.sub.js

var test = async_test();

var wsocket = CreateWebSocket(false, false, false);
var isOpenCalled = false;

wsocket.addEventListener('open', test.step_func(function(evt) {
  wsocket.close(undefined);
  test.done();
}), true);
