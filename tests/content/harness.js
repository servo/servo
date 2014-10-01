function _oneline(x) {
  var i = x.indexOf("\n");
  return (i == -1) ? x : (x.slice(0, i) + "...");
}

var _expectations = 0;
var _tests = 0;
function expect(num) {
  _expectations = num;
}

function _fail(s, m) {
  _tests++;
  // string split to avoid problems with tests that end up printing the value of window._fail.
  console.log(_oneline("TEST-UNEXPECTED" + "-FAIL | " + s + ": " + m));
}

function _pass(s, m) {
  _tests++;
  //console.log(_oneline("TEST-PASS | " + s + ": " + m));
}

function _printer(opstr, op) {
  return function (a, b, msg) {
    var f = op(a,b) ? _pass : _fail;
    if (!msg) msg = "";
    f(a + " " + opstr + " " + b, msg);
  };
}

var is          = _printer("===",          function (a,b) { return a === b; });
var is_not      = _printer("!==",          function (a,b) { return a !== b; });
var is_a        = _printer("is a",         function (a,b) { return a instanceof b; });
var is_not_a    = _printer("is not a",     function (a,b) { return !(a instanceof b); });
var is_in       = _printer("is in",        function (a,b) { return a in b; });
var is_not_in   = _printer("is not in",    function (a,b) { return !(a in b); });
var as_str_is   = _printer("as string is", function (a,b) { return String(a) == b; });
var lt          = _printer("<",            function (a,b) { return a <  b; });
var gt          = _printer(">",            function (a,b) { return a >  b; });
var leq         = _printer("<=",           function (a,b) { return a <= b; });
var geq         = _printer(">=",           function (a,b) { return a >= b; });
var starts_with = _printer("starts with",  function (a,b) { return a.indexOf(b) == 0; });

function is_function(val, name) {
  starts_with(String(val), "function " + name + "(");
}

function should_throw(f) {
  try {
    f();
    _fail("operation should have thrown but did not");
  } catch (x) {
    _pass("operation successfully threw an exception", x.toString());
  }
}

function should_not_throw(f) {
  try {
    f();
    _pass("operation did not throw an exception");
  } catch (x) {
    _fail("operation should have not thrown", x.toString());
  }
}

function check_selector(elem, selector, matches) {
    is(elem.matches(selector), matches);
}

function check_disabled_selector(elem, disabled) {
    check_selector(elem, ":disabled", disabled);
    check_selector(elem, ":enabled", !disabled);
}

var _test_complete = false;
var _test_timeout = 10000; //10 seconds
function finish() {
   if (_test_complete) {
    _fail('finish called multiple times');
  }
  if (_expectations > _tests) {
    _fail('expected ' + _expectations + ' tests, fullfilled ' + _tests);
  }
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

var _needs_finish = false;
function waitForExplicitFinish() {
    _needs_finish = true;
}

addEventListener('load', function() {
  if (!_needs_finish) {
    finish();
  }
});
