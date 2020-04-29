// META: title=Cookie Store API: cookieListItem attributes
// META: global=window,serviceworker

'use strict';

const kCurrentHostname = (new URL(self.location.href)).hostname;

const kOneDay = 24 * 60 * 60 * 1000;
const kTenYears = 10 * 365 * kOneDay;
const kTenYearsFromNow = Date.now() + kTenYears;

const kCookieListItemKeys =
    ['domain', 'expires', 'name', 'path', 'sameSite', 'secure', 'value'].sort();

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set defaults with positional name and value');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set defaults with name and value in options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value',
                        { expires: kTenYearsFromNow });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with expires set to a timestamp 10 ' +
   'years in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: kTenYearsFromNow });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with name and value in options and ' +
   'expires set to a future timestamp');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value',
                        { expires: new Date(kTenYearsFromNow) });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);
}, 'CookieListItem - cookieStore.set with expires set to a Date 10 ' +
   'years in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: new Date(kTenYearsFromNow) });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_approx_equals(cookie.expires, kTenYearsFromNow, kOneDay);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with name and value in options and ' +
   'expires set to a future Date');

promise_test(async testCase => {
  await cookieStore.delete({ name: 'cookie-name', domain: kCurrentHostname });

  await cookieStore.set('cookie-name', 'cookie-value',
                        { domain: kCurrentHostname });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: kCurrentHostname });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, kCurrentHostname);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set('cookie-name', 'cookie-value',
                        { path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, currentDirectory);
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory = currentPath.substr(0, currentPath.lastIndexOf('/'));
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set('cookie-name', 'cookie-value',
                        { path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, currentDirectory + '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, true);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set adds / to path if it does not end with /');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value', { secure: false });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.domain, null);
  assert_equals(cookie.path, '/');
  assert_equals(cookie.expires, null);
  assert_equals(cookie.secure, false);
  assert_equals(cookie.sameSite, 'strict');
  assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
}, 'CookieListItem - cookieStore.set with secure set to false');

['strict', 'lax', 'none'].forEach(sameSiteValue => {
  promise_test(async testCase => {
    await cookieStore.delete('cookie-name');

    await cookieStore.set({
        name: 'cookie-name', value: 'cookie-value', sameSite: sameSiteValue });
    testCase.add_cleanup(async () => {
      await cookieStore.delete('cookie-name');
    });
    const cookie = await cookieStore.get('cookie-name');
    assert_equals(cookie.name, 'cookie-name');
    assert_equals(cookie.value, 'cookie-value');
    assert_equals(cookie.domain, null);
    assert_equals(cookie.path, '/');
    assert_equals(cookie.expires, null);
    assert_equals(cookie.secure, true);
    assert_equals(cookie.sameSite, sameSiteValue);
    assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
  }, `CookieListItem - cookieStore.set with sameSite set to ${sameSiteValue}`);

  promise_test(async testCase => {
    await cookieStore.delete('cookie-name');

    await cookieStore.set('cookie-name', 'cookie-value',
                          { sameSite: sameSiteValue });
    testCase.add_cleanup(async () => {
      await cookieStore.delete('cookie-name');
    });
    const cookie = await cookieStore.get('cookie-name');
    assert_equals(cookie.name, 'cookie-name');
    assert_equals(cookie.value, 'cookie-value');
    assert_equals(cookie.domain, null);
    assert_equals(cookie.path, '/');
    assert_equals(cookie.expires, null);
    assert_equals(cookie.secure, true);
    assert_equals(cookie.sameSite, sameSiteValue);
    assert_array_equals(Object.keys(cookie).sort(), kCookieListItemKeys);
  }, 'CookieListItem - cookieStore.set with positional name and value and ' +
     `sameSite set to ${sameSiteValue}`);
});
