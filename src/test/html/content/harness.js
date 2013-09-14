function _fail(s) {
  // string split to avoid problems with tests that end up printing the value of window._fail.
  window.alert("TEST-UNEXPECTED" + "-FAIL | " + s);
}

function _pass(s) {
  window.alert("TEST-PASS | " + s);
}

function is(a, b) {
  let f = a != b ? _fail : _pass;
  f(a + " == " + b);
}

var _test_complete = false;
var _test_timeout = 10000; //10 seconds
function finish() {
  _test_complete = true;
  window.close();
}

function _test_timed_out() {
  if (!_test_complete) {
    _fail('test timed out (' + _test_timeout/1000 + 's)');
    finish();
  }
}

setTimeout(_test_timed_out, _test_timeout);