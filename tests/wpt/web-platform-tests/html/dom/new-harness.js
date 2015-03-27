// We override only the things we need to -- the rest we'll just inherit from
// original-harness.js.  Polymorphism, kind of.
ReflectionHarness.catchUnexpectedExceptions = false;

ReflectionHarness.test = function(expected, actual, description) {
  test(function() {
    assert_equals(expected, actual);
  }, this.getTypeDescription() + ": " + description);
  // This is the test suite that will rate conformance, so we don't want to
  // bail out early if a test fails -- we want all tests to always run.
  return true;
}

ReflectionHarness.run = function(fun, description) {
  test(fun, this.getTypeDescription() + ": " + description);
}

ReflectionHarness.testException = function(exceptionName, fn, description) {
  test(function() {
    assert_throws(exceptionName, fn);
  }, this.getTypeDescription() + ": " + description);
}
