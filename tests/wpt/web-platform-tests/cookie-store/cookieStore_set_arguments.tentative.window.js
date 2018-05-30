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
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with positional name and value');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with name and value in options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await promise_rejects(testCase, new TypeError(), cookieStore.set(
      'cookie-name', 'cookie-value', { name: 'cookie-name' }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await promise_rejects(testCase, new TypeError(), cookieStore.set(
      'cookie-name', 'cookie-value', { value: 'cookie-value' }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with value in both positional arguments and options');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: tenYearsFromNow });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires in the future');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: tenYearsAgo });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with expires in the past');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', expires: tenYearsFromNow });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with name and value in options and expires in the future');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', expires: tenYearsAgo });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.set with name and value in options and expires in the past');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.delete('cookie-name', { domain: currentDomain });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: currentDomain });
  });
}, 'cookieStore.set with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;
  await cookieStore.delete('cookie-name', { domain: currentDomain });
  await cookieStore.delete('cookie-name', { domain: subDomain });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: subDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: subDomain });
  });
}, 'cookieStore.set with domain set to a subdomain of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-old-value');
  await cookieStore.set(
      'cookie-name', 'cookie-new-value', { domain: currentDomain });

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-new-value');

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name');
    await cookieStore.delete('cookie-name', { domain: currentDomain });
  });
}, 'cookieStore.set default domain is current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.delete('cookie-name', { path: currentDirectory });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: currentDirectory });
  });
}, 'cookieStore.set with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const subDirectory = currentDirectory + "subdir/";
  await cookieStore.delete('cookie-name', { path: currentDirectory });
  await cookieStore.delete('cookie-name', { path: subDirectory });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: subDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: subDirectory });
  });
}, 'cookieStore.set with path set to a subdirectory of the current directory');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-old-value');
  await cookieStore.set('cookie-name', 'cookie-new-value', { path: '/' });

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-new-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
  await async_cleanup(() => cookieStore.delete('cookie-name', { path: '/' }));
}, 'cookieStore.set default path is /');
