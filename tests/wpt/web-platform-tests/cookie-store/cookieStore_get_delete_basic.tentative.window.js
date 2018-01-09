'use strict';

// Workaround because add_cleanup doesn't support async functions yet.
// See https://github.com/w3c/web-platform-tests/issues/6075
async function async_cleanup(cleanup_function) {
  try {
    await cleanup_function();
  } catch (e) {
    // Errors in cleanup functions shouldn't result in test failures.
  }
}

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.delete('cookie-name');
  const cookie = await cookieStore.get();
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get returns null for a cookie deleted by cookieStore.delete');