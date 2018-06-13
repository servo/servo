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
  const inTwentyFourHours = new Date(Date.now() + 24 * 60 * 60 * 1000);

  assert_equals(
    await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: inTwentyFourHours }),
    undefined);

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires option: Date object');

promise_test(async testCase => {
  const inTwentyFourHours = Date.now() + 24 * 60 * 60 * 1000;

  assert_equals(
    await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: inTwentyFourHours }),
    undefined);

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires option: milliseconds since epoch object');

promise_test(async testCase => {
  const year = (new Date()).getUTCFullYear() + 1;
  const date = new Date('07 Jun ' + year + ' 07:07:07 UTC');
  const day = ('Sun Mon Tue Wed Thu Fri Sat'.split(' '))[date.getUTCDay()];
  const nextJune = `${day}, 07 Jun ${year} + ' 07:07:07 GMT`;

  assert_equals(
    await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: nextJune }),
    undefined);

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires option: HTTP date string');
