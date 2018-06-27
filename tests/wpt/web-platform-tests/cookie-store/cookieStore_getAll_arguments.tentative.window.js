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

  const cookies = await cookieStore.getAll();
  cookies.sort((a, b) => a.name.localeCompare(b.name));
  assert_equals(cookies.length, 2);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');
  assert_equals(cookies[1].name, 'cookie-name-2');
  assert_equals(cookies[1].value, 'cookie-value-2');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with no arguments');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  const cookies = await cookieStore.getAll({ name: 'cookie-name' });
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  await promise_rejects(testCase, new TypeError(), cookieStore.get(
      'cookie-name', { name: 'cookie-name' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const cookies = await cookieStore.getAll(
      'cookie-name', { matchType: 'equals' });
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-value');

  const no_cookies = await cookieStore.getAll(
      'cookie-na', { matchType: 'equals' });
  assert_equals(no_cookies.length, 0);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.getAll with matchType explicitly set to equals');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  const cookies = await cookieStore.getAll(
      'cookie-name-', { matchType: 'starts-with' });
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name-2');
  assert_equals(cookies[0].value, 'cookie-value-2');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with matchType set to starts-with');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  await promise_rejects(testCase, new TypeError(), cookieStore.getAll(
      'cookie-name', { matchType: 'invalid' }));

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with invalid matchType');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  await cookieStore.set('cookie-name-2', 'cookie-value-2');

  const cookies = await cookieStore.getAll(
      { matchType: 'starts-with', name: 'cookie-name-' });
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name-2');
  assert_equals(cookies[0].value, 'cookie-value-2');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name-2'));
}, 'cookieStore.getAll with matchType set to starts-with and name in options');
