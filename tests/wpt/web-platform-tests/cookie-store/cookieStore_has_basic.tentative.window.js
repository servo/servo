'use strict';

// Workaround because add_cleanup doesn't support async functions yet.
// See https://github.com/web-platform-tests/wpt/issues/6075
async function async_cleanup(cleanup_function) {
  try {
    await cleanup_function();
  } catch (e) {
    // Errors in cleanup functions shouldn't result in test failures.
  }
}

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  assert_equals(await cookieStore.has('cookie-name'), true);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has returns true for cookie set by cookieStore.set()');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');
  assert_equals(await cookieStore.has('cookie-name'), false);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has returns false for cookie deleted by cookieStore.delete()');
