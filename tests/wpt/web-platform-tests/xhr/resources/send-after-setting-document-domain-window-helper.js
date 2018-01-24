function assert_equals(value, expected) {
  if (value != expected) {
    throw "Got wrong value.\nExpected '" + expected + "',\ngot '" + value + "'";
  }
}

function assert_throws(expected_exc, func) {
  try {
    func.call(this);
  } catch(e) {
    var actual = e.name || e.type;
    if (actual != expected_exc) {
      throw "Got wrong exception.\nExpected '" + expected_exc + "',\ngot '" + actual + "'.";
    }
    return;
  }
  throw "Expected exception, but none was thrown";
}

function run_test(test, name) {
  var result = {passed: true, message: null, name: name};
  try {
    test();
  } catch(e) {
    result.passed = false;
    result.message = e + "";
  }
  opener.postMessage(result, "*");
}
