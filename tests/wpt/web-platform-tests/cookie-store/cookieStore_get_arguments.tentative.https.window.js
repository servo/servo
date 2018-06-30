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

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get({ name: 'cookie-name' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.get(
      'cookie-name', { name: 'cookie-name' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get(
      'cookie-name', { matchType: 'equals' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  const no_cookie = await cookieStore.get(
      'cookie-na', { matchType: 'equals' });
  assert_equals(no_cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with matchType explicitly set to equals');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get(
      'cookie-na', { matchType: 'starts-with' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with matchType set to starts-with');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.get(
      'cookie-name', { matchType: 'invalid' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with invalid matchType');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get(
      { matchType: 'starts-with', name: 'cookie-na' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.get with matchType set to starts-with and name in options');
