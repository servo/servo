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
  await cookieStore.delete('cookie-name-2');

  const has_cookie = await cookieStore.has('cookie-name');
  assert_equals(has_cookie, true);
  const has_cookie2 = await cookieStore.has('cookie-name-2');
  assert_equals(has_cookie2, false);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.delete('cookie-name-2');

  const has_cookie = await cookieStore.has({ name: 'cookie-name' });
  assert_equals(has_cookie, true);
  const has_cookie2 = await cookieStore.has({ name: 'cookie-name-2' });
  assert_equals(has_cookie2, false);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.has(
      'cookie-name', { name: 'cookie-name' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const has_cookie = await cookieStore.has(
      'cookie-na', { matchType: 'equals' });
  assert_equals(has_cookie, false);
  const has_cookie2 = await cookieStore.has(
      'cookie-name', { matchType: 'equals' });
  assert_equals(has_cookie2, true);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with matchType explicitly set to equals');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const has_cookie = await cookieStore.has(
      'cookie-na', { matchType: 'startsWith' });
  assert_equals(has_cookie, true);
  const has_cookie2 = await cookieStore.has(
      'cookie-name-', { matchType: 'startsWith' });
  assert_equals(has_cookie2, false);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with matchType set to startsWith');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.has(
      'cookie-name', { matchType: 'invalid' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with invalid matchType');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const has_cookie = await cookieStore.has(
      { matchType: 'startsWith', name: 'cookie-na' });
  assert_equals(has_cookie, true);
  const has_cookie2 = await cookieStore.has(
      { matchType: 'startsWith', name: 'cookie-name-' });
  assert_equals(has_cookie2, false);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.has with matchType set to startsWith and name in options');
