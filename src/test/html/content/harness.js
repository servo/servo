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

function finish() {
  window.close();
}