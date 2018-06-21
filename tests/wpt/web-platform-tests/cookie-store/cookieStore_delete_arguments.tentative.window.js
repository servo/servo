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

  await cookieStore.delete('cookie-name');
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await cookieStore.delete({ name: 'cookie-name' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.delete with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      'cookie-name', { name: 'cookie-name' }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.delete with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      'cookie-name', { value: 'cookie-value' }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.delete with value in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      'cookie-name', { expires: tenYearsAgo }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore.delete with expires in options');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });

  await cookieStore.delete('cookie-name', { domain: currentDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: currentDomain })
  });
}, 'cookieStore.delete with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      'cookie-name', 'cookie-value', { domain: subDomain }));
}, 'cookieStore.delete with domain set to a subdomain of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      'cookie-name', { domain: domainSuffix }));
}, 'cookieStore.delete with domain set to a non-domain-matching suffix of ' +
   'the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });

  await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { domain: currentDomain })
  });
}, 'cookieStore.delete with name in options and domain set to the current ' +
   'hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      { name: 'cookie-name', domain: subDomain }));
}, 'cookieStore.delete with name in options and domain set to a subdomain of ' +
   'the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects(testCase, new TypeError(), cookieStore.delete(
      { name: 'cookie-name', domain: domainSuffix }));
}, 'cookieStore.delete with name in options and domain set to a ' +
   'non-domain-matching suffix of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory });

  await cookieStore.delete('cookie-name', { path: currentDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);

  async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: currentDirectory })
  });
}, 'cookieStore.delete with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const subDirectory = currentDirectory + "subdir/";
  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory });

  await cookieStore.delete('cookie-name', { path: subDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  await async_cleanup(async () => {
    await cookieStore.delete('cookie-name', { path: currentDirectory })
  });
}, 'cookieStore.delete with path set to subdirectory of the current directory');
