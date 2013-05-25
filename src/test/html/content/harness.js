function _fail(s) {
  window.alert("TEST-UNEXPECTED-FAIL | " + s);
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