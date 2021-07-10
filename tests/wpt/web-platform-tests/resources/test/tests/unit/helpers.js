// Helper for testing assertion failure cases for a testharness.js API
//
// The `assert_throws_*` functions cannot be used for this purpose because they
// always fail in response to AssertionError exceptions, even when this is
// expressed as the expected error.
function test_failure(fn, name) {
  test(function() {
    try {
      fn();
    } catch (err) {
      if (err instanceof AssertionError) {
        return;
      }
      throw new AssertionError('Expected an AssertionError, but' + err);
    }
    throw new AssertionError(
      'Expected an AssertionError, but no error was thrown'
    );
  }, name);
}

