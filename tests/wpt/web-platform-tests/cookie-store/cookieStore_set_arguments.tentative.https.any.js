// META: title=Cookie Store API: cookieStore.set() arguments
// META: global=!default,serviceworker,window

'use strict';

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with positional name and value');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set({ name: 'cookie-name', value: 'cookie-value' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with name and value in options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  cookieStore.set('cookie-name', 'cookie-value', { name: 'wrong-cookie-name' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  cookieStore.set('cookie-name', 'cookie-value',
                  { value: 'wrong-cookie-value' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with value in both positional arguments and options');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: tenYearsFromNow });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with expires set to a future timestamp');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: tenYearsAgo });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with expires set to a past timestamp');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: new Date(tenYearsFromNow) });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with expires set to a future Date');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      'cookie-name', 'cookie-value', { expires: new Date(tenYearsAgo) });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with expires set to a past Date');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', expires: tenYearsFromNow });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with name and value in options and expires in the future');

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsAgo = Date.now() - tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', expires: tenYearsAgo });
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with name and value in options and expires in the past');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { domain: currentDomain });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with domain set to the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  const subDomain = `sub.${currentDomain}`;

  await promise_rejects(testCase, new TypeError(), cookieStore.set(
      'cookie-name', 'cookie-value', { domain: subDomain }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with domain set to a subdomain of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects(testCase, new TypeError(), cookieStore.set(
      'cookie-name', 'cookie-value', { domain: domainSuffix }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with domain set to a non-domain-matching suffix of the ' +
   'current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-value1');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  await cookieStore.set('cookie-name', 'cookie-value2',
                        { domain: currentDomain });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });
  });

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 2);

  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[1].name, 'cookie-name');

  const values = cookies.map((cookie) => cookie.value);
  values.sort();
  assert_array_equals(values, ['cookie-value1', 'cookie-value2']);
}, 'cookieStore.set default domain is null and differs from current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.set with path set to the current directory');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const subDirectory = currentDirectory + "subdir/";
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  await cookieStore.delete({ name: 'cookie-name', path: subDirectory });

  await cookieStore.set(
      'cookie-name', 'cookie-value', { path: subDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: subDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with path set to a subdirectory of the current directory');

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');

  await cookieStore.set('cookie-name', 'cookie-old-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  await cookieStore.set('cookie-name', 'cookie-new-value', { path: '/' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name',  path: '/' });
  });

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-new-value');
}, 'cookieStore.set default path is /');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'old-cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie_attributes = await cookieStore.get('cookie-name');
  assert_equals(cookie_attributes.name, 'cookie-name');
  assert_equals(cookie_attributes.value, 'old-cookie-value');

  cookie_attributes.value = 'new-cookie-value';
  await cookieStore.set(cookie_attributes);
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'new-cookie-value');
}, 'cookieStore.set with get result');
