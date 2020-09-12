var t = async_test('Test behavior of sync-script feature policy for different script types');

window.onload = t.step_func(function() {
  assert_equals(undefined, window.didExecuteInlineParsingBlockingScript, 'inline parser blocking script should be blocked');
  assert_equals(undefined, window.didExecuteExternalParsingBlockingScript, 'external parser blocking script should be blocked');
  assert_true(window.didExecuteExternalAsyncScript, 'external async script should not be blocked');
  assert_true(window.didExecuteExternalDeferredScript, 'external defer script should not be blocked');
  t.done();
});
