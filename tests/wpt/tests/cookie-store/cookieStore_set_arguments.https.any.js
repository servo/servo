// META: title=Cookie Store API: cookieStore.set() arguments
// META: global=window,serviceworker

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
  await promise_rejects_js(testCase, TypeError,
      cookieStore.set('', 'suspicious-value=resembles-name-and-value'));
}, "cookieStore.set with empty name and an '=' in value");

promise_test(async testCase => {
  await cookieStore.delete('cookie-name');
  cookieStore.set('cookie-name', 'suspicious-value=resembles-name-and-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'suspicious-value=resembles-name-and-value');
}, "cookieStore.set with normal name and an '=' in value");

promise_test(async testCase => {
  const tenYears = 10 * 365 * 24 * 60 * 60 * 1000;
  const tenYearsFromNow = Date.now() + tenYears;
  await cookieStore.delete('cookie-name');

  await cookieStore.set(
      { name: 'cookie-name',
        value: 'cookie-value',
        expires: new Date(tenYearsFromNow) });
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
      { name :'cookie-name',
        value: 'cookie-value',
        expires: new Date(tenYearsAgo) });
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
}, 'cookieStore.set with expires set to a future timestamp');

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
}, 'cookieStore.set with expires set to a past timestamp');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name',
        value: 'cookie-value',
        domain: `.${currentDomain}` }));
}, 'cookieStore.set domain starts with "."');

promise_test(async testCase => {
  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', domain: 'example.com' }));
}, 'cookieStore.set with domain that is not equal current host');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  await cookieStore.delete({ name: 'cookie-name', domain: currentDomain });

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', domain: currentDomain });
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

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', domain: subDomain }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set with domain set to a subdomain of the current hostname');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentDomain = currentUrl.hostname;
  assert_not_equals(currentDomain[0] === '.',
      'this test assumes that the current hostname does not start with .');
  const domainSuffix = currentDomain.substr(1);

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', domain: domainSuffix }));
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
  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value2', domain: currentDomain });
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
      { name: 'cookie-name', value: 'cookie-value', path: currentDirectory });
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
      { name: 'cookie-name', value: 'cookie-value', path: subDirectory });
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
  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-new-value', path: '/' });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name',  path: '/' });
  });

  const cookies = await cookieStore.getAll('cookie-name');
  assert_equals(cookies.length, 1);
  assert_equals(cookies[0].name, 'cookie-name');
  assert_equals(cookies[0].value, 'cookie-new-value');
}, 'cookieStore.set default path is /');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory = currentPath.substr(0, currentPath.lastIndexOf('/'));
  await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });

  await cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', path: currentDirectory });
  testCase.add_cleanup(async () => {
    await cookieStore.delete({ name: 'cookie-name', path: currentDirectory });
  });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
  assert_equals(cookie.path, currentDirectory + '/');
}, 'cookieStore.set adds / to path that does not end with /');

promise_test(async testCase => {
  const currentUrl = new URL(self.location.href);
  const currentPath = currentUrl.pathname;
  const currentDirectory =
      currentPath.substr(0, currentPath.lastIndexOf('/') + 1);
  const invalidPath = currentDirectory.substr(1);

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name', value: 'cookie-value', path: invalidPath }));
}, 'cookieStore.set with path that does not start with /');

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

promise_test(async testCase => {
  // The maximum attribute value size is specified as 1024 bytes at https://wicg.github.io/cookie-store/#cookie-maximum-attribute-value-size.
  await cookieStore.delete('cookie-name');

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name',
        value: 'cookie-value',
        path: '/' + 'a'.repeat(1023) + '/' }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set checks if the path is too long');

promise_test(async testCase => {
  // The maximum attribute value size is specified as 1024 bytes at https://wicg.github.io/cookie-store/#cookie-maximum-attribute-value-size.
  await cookieStore.delete('cookie-name');

  await promise_rejects_js(testCase, TypeError, cookieStore.set(
      { name: 'cookie-name',
        value: 'cookie-value',
        domain: 'a'.repeat(1025) }));
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.set checks if the domain is too long');
