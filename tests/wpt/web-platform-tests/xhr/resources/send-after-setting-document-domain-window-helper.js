function assert_equals(value, expected) {
  if (value != expected) {
    throw "Got wrong value.\nExpected '" + expected + "',\ngot '" + value + "'";
  }
}

function assert_throws_dom(expected_exc, func) {
  try {
    func.call(this);
  } catch(e) {
    if (e.constructor.name != "DOMException") {
      throw `Exception ${e.constructor.name || "unknown"} that was not a DOMException was thrown`;
    }
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
