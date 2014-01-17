function _oneline(x) {
  var i = x.indexOf("\n");
  return (i == -1) ? x : (x.slice(0, i) + "...");
}

function _fail(s, m) {
  // string split to avoid problems with tests that end up printing the value of window._fail.
  window.alert(_oneline("TEST-UNEXPECTED" + "-FAIL | " + s + ": " + m));
}

function _pass(s, m) {
  window.alert(_oneline("TEST-PASS | " + s + ": " + m));
}

function _printer(opstr, op) {
  return function (a, b, msg) {
    let f = op(a,b) ? _pass : _fail;
    if (!msg) msg = "";
    f(a + " " + opstr + " " + b, msg);
  };
}

var is          = _printer("==",           function (a,b) { return a == b; });
var is_a        = _printer("is a",         function (a,b) { return a instanceof b; });
var is_not_a    = _printer("is not a",     function (a,b) { return !(a instanceof b); });
var is_in       = _printer("is in",        function (a,b) { return a in b; });
var is_not_in   = _printer("is not in",    function (a,b) { return !(a in b); });
var as_str_is   = _printer("as string is", function (a,b) { return String(a) == b; });
var isnot       = _printer("!=",           function (a,b) { return a != b; });
var lt          = _printer("<",            function (a,b) { return a <  b; });
var gt          = _printer(">",            function (a,b) { return a >  b; });
var leq         = _printer("<=",           function (a,b) { return a <= b; });
var geq         = _printer(">=",           function (a,b) { return a >= b; });
var starts_with = _printer("starts with",  function (a,b) { return a.indexOf(b) == 0; });

function is_function(val, name) {
  starts_with(String(val), "function " + name + "(");
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
