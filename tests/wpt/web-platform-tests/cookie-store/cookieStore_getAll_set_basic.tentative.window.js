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
  const cookies = await cookieStore.getAll('cookie-name');

  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.getAll returns the cookie written by cookieStore.set');
