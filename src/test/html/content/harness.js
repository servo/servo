function _fail(s, m) {
  // string split to avoid problems with tests that end up printing the value of window._fail.
  window.alert("TEST-UNEXPECTED" + "-FAIL | " + s + ": " + m);
}

function _pass(s, m) {
  window.alert("TEST-PASS | " + s + ": " + m);
}

function is(a, b, c) {
  let f = a != b ? _fail : _pass;
  let m = !c ? "" : c;
  f(a + " == " + b, m);
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
