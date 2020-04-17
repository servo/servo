// META: title=Cookie Store API: cookieStore.get() arguments
// META: global=!default,serviceworker,window

'use strict';

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get();
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with no arguments');

promise_test(async testCase => {
  await cookieStore.set('cookie-name-1', 'cookie-value-1');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name-1');
  });
  await cookieStore.set('cookie-name-2', 'cookie-value-2');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name-2');
  });

  const cookie = await cookieStore.get();
  assert_equals(cookie.name, 'cookie-name-1');
  assert_equals(cookie.value, 'cookie-value-1');
},'cookieStore.get with no args and multiple matches');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with positional name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get({ name: 'cookie-name' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with name in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get('cookie-name',
                                       { name: 'wrong-cookie-name' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with name in both positional arguments and options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get(
      'cookie-name', { matchType: 'equals' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');

  const no_cookie = await cookieStore.get({ name: 'cookie-na',
                                            matchType: 'equals' });
  assert_equals(no_cookie, null);
}, 'cookieStore.get with matchType explicitly set to equals');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get({ name: 'cookie-na',
                                         matchType: 'starts-with' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with matchType set to starts-with');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  await promise_rejects_js(testCase, TypeError, cookieStore.get(
      { name: 'cookie-name', matchType: 'invalid' }));
}, 'cookieStore.get with invalid matchType');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get({ matchType: 'equals' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with matchType set to equals and missing name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  const cookie = await cookieStore.get({ matchType: 'starts-with' });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with matchType set to starts-with and missing name');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  let target_url = self.location.href;
  if (self.GLOBAL.isWorker()) {
    target_url = target_url + '/path/within/scope';
  }

  const cookie = await cookieStore.get({ url: target_url });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with absolute url in options');

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
    await cookieStore.delete('cookie-name');
  });

  let target_path = self.location.pathname;
  if (self.GLOBAL.isWorker()) {
    target_path = target_path + '/path/within/scope';
  }

  const cookie = await cookieStore.get({ url: target_path });
  assert_equals(cookie.name, 'cookie-name');
  assert_equals(cookie.value, 'cookie-value');
}, 'cookieStore.get with relative url in options');

promise_test(async testCase => {
  const invalid_url =
      `${self.location.protocol}//${self.location.host}/different/path`;
  await promise_rejects_js(testCase, TypeError, cookieStore.get(
      { url: invalid_url }));
}, 'cookieStore.get with invalid url path in options');

promise_test(async testCase => {
  const invalid_url =
      `${self.location.protocol}//www.example.com${self.location.pathname}`;
  await promise_rejects_js(testCase, TypeError, cookieStore.get(
      { url: invalid_url }));
}, 'cookieStore.get with invalid url host in options');
