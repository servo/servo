// META: title=Cookie Store API: cookieListItem attributes
// META: global=serviceworker

// This is a copy of cookieListItem_attributes.https.window.js, minus all the bits that only work
// in a window context, which unfortunately includes the testdriver asserts.
//
// Please keep them synchronized.

'use strict';

const currentHostname = (new URL(self.location.href)).hostname;

const oneDayInSeconds = 24 * 60 * 60;
const fourHundredDaysInSeconds = 400 * oneDayInSeconds;
const tenYearsInSeconds = 10 * 365 * oneDayInSeconds;
const fourHundredDaysFromNowInSeconds = Date.now() / 1000 + fourHundredDaysInSeconds;
const tenYearsFromNowInSeconds = Date.now() / 1000 + tenYearsInSeconds;

function assert_cookie_keys(cookie) {
  const cookieKeys = Object.keys(cookie);
  assert_array_equals(cookieKeys, ['name', 'value']);
}

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
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
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set defaults with name and value in options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: tenYearsFromNowInSeconds * 1000 });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set with expires set to a timestamp 10 ' +
   'years in the future');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          expires: new Date(tenYearsFromNowInSeconds) * 1000 });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set with expires set to a Date 10 ' +
   'years in the future');

promise_test(async testCase => {
  await cookieStore.delete({ name: 'cookie-name', domain: currentHostname });

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          domain: currentHostname });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: currentHostname });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory = currentPath.substr(0, currentPath.lastIndexOf('/'));
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value',
                          path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_cookie_keys(cookie);
}, 'CookieListItem - cookieStore.set does not add / to path if it does not end with /');

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
    assert_cookie_keys(cookie);
  }, `CookieListItem - cookieStore.set with sameSite set to ${sameSiteValue}`);

});
