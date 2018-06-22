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

const kCurrentHostname = (new URL(self.location.href)).hostname;
const kIsSecureTransport = (new URL(self.location.href)).protocol === 'https:';

const kOneDay = 24 * 60 * 60 * 1000;
const kTenYears = 10 * 365 * kOneDay;
const kTenYearsFromNow = Date.now() + kTenYears;

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set defaults with positional name and value');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set defaults with name and value in options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value',
                        { expires: kTenYearsFromNow });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires set to 10 years in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: kTenYearsFromNow });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with name and value in options and expires in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name', { domain: kCurrentHostname });

  await cookieStore.set('cookie-name', 'cookie-value',
                        { domain: kCurrentHostname });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, kCurrentHostname);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: kCurrentHostname });
  });
}, 'cookieStore.set with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.delete('cookie-name', { path: currentDirectory });

  await cookieStore.set('cookie-name', 'cookie-value',
                        { path: currentDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, currentDirectory);
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, kIsSecureTransport);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: currentDirectory });
  });
}, 'cookieStore.set with path set to the current directory');
