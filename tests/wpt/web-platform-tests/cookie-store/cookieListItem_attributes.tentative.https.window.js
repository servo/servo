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

const kOneDay = 24 * 60 * 60 * 1000;
const kTenYears = 10 * 365 * kOneDay;
const kTenYearsFromNow = Date.now() + kTenYears;

const kCookieListItemKeys =
    ['domain', 'expires', 'name', 'path', 'sameSite', 'secure', 'value'].sort();

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set defaults with positional name and value');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set defaults with name and value in options');

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
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set with expires set to a timestamp 10 ' +
   'years in the future');

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
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set with name and value in options and ' +
   'expires set to a future timestamp');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value',
                        { expires: new Date(kTenYearsFromNow) });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set with expires set to a Date 10 ' +
   'years in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: new Date(kTenYearsFromNow) });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'CookieListItem - cookieStore.set with name and value in options and ' +
   'expires set to a future Date');

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
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: kCurrentHostname });
  });
}, 'CookieListItem - cookieStore.set with domain set to the current hostname');

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
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: currentDirectory });
  });
}, 'CookieListItem - cookieStore.set with path set to the current directory');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name', { secure: false });

  await cookieStore.set('cookie-name', 'cookie-value', { secure: false });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, false);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { secure: false });
  });
}, 'CookieListItem - cookieStore.set with secure set to false');

['strict', 'lax', 'unrestricted'].forEach(sameSiteValue => {
  promise_test(async testCase => {
    await cookieStore.delete('cookie-name', { sameSite: sameSiteValue });

    await cookieStore.set({
        name: 'cookie-name', value: 'cookie-value', sameSite: sameSiteValue });
    const cookie = await cookieStore.get('cookie-name');
    assert_equals(cookie.name, 'cookie-name');
    assert_equals(cookie.value, 'cookie-value');
    assert_equals(cookie.domain, null);
    assert_equals(cookie.path, '/');
    assert_equals(cookie.expires, null);
    assert_equals(cookie.secure, true);
    assert_equals(cookie.sameSite, sameSiteValue);
    assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

    await async_cleanup(async () => {
      await cookieStore.delete('cookie-name', { secure: sameSiteValue });
    });
  }, `CookieListItem - cookieStore.set with sameSite set to ${sameSiteValue}`);

  promise_test(async testCase => {
    await cookieStore.delete('cookie-name', { sameSite: sameSiteValue });

    await cookieStore.set('cookie-name', 'cookie-value',
                          { sameSite: sameSiteValue });
    const cookie = await cookieStore.get('cookie-name');
    assert_equals(cookie.name, 'cookie-name');
    assert_equals(cookie.value, 'cookie-value');
    assert_equals(cookie.domain, null);
    assert_equals(cookie.path, '/');
    assert_equals(cookie.expires, null);
    assert_equals(cookie.secure, true);
    assert_equals(cookie.sameSite, sameSiteValue);
    assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);

    await async_cleanup(async () => {
      await cookieStore.delete('cookie-name', { secure: sameSiteValue });
    });
  }, 'CookieListItem - cookieStore.set with positional name and value and ' +
     `sameSite set to ${sameSiteValue}`);
});
