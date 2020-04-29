// META: title=Cookie Store API: cookieStore.delete() arguments
// META: global=window,serviceworker

'use strict';

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  await cookieStore.delete('cookie-name');
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  await cookieStore.delete({ name: 'cookie-name' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  await cookieStore.delete('cookie-name', { name: 'wrong-cookie-name' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with name in both positional arguments and options');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  });

  await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;

  await promise_rejects_js(testCase, TypeError, cookieStore.delete(
      { name: 'cookie-name', domain: subDomain }));
}, 'cookieStore.delete with domain set to a subdomain of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects_js(testCase, TypeError, cookieStore.delete(
      { name: 'cookie-name', domain: domainSuffix }));
}, 'cookieStore.delete with domain set to a non-domain-matching suffix of ' +
   'the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  });

  await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with name in options and domain set to the current ' +
   'hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;

  await promise_rejects_js(testCase, TypeError, cookieStore.delete(
      { name: 'cookie-name', domain: subDomain }));
}, 'cookieStore.delete with name in options and domain set to a subdomain of ' +
   'the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects_js(testCase, TypeError, cookieStore.delete(
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
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });

  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const subDirectory = currentDirectory + "subdir/";
  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });

  await cookieStore.delete({ name: 'cookie-name', path: subDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.delete with path set to subdirectory of the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory = currentPath.substr(0, currentPath.lastIndexOf('/'));
  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory + '/' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });

  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with missing / at the end of path');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const invalidPath = currentDirectory.substr(1);

  await promise_rejects_js(testCase, TypeError, cookieStore.delete(
      { name: 'cookie-name', path: invalidPath }));
}, 'cookieStore.delete with path that does not start with /');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie_attributes = await cookieStore.get('cookie-name');
  assert_equals(cookie_attributes.name, 'cookie-name');
  assert_equals(cookie_attributes.value, 'cookie-value');

  await cookieStore.delete(cookie_attributes);
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete with get result');
