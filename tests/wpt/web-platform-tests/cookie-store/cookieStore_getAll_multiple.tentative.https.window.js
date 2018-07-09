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
  await cookieStore.set('cookie-name-2', 'cookie-value-2');
  await cookieStore.set('cookie-name-3', 'cookie-value-3');

  const cookies = await cookieStore.getAll();
  cookies.sort((a, b) => a.name.localeCompare(b.name));
  assert_equals(cookies.length, 3);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');
  assert_equals(cookies[1].name, 'cookie-name-2');
  assert_equals(cookies[1].value, 'cookie-value-2');
  assert_equals(cookies[2].name, 'cookie-name-3');
  assert_equals(cookies[2].value, 'cookie-value-3');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
  await async_cleanup(() => cookieStore.delete('cookie-name-3'));
}, 'cookieStore.getAll returns multiple cookies written by cookieStore.set');
