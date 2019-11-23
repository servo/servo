// META: title=Cookie Store API: Interaction between cookieStore.set() and cookieStore.delete()
// META: global=!default,serviceworker,window

'use strict';

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');
  testCase.add_cleanup(async () => {
     await cookieStore.delete('cookie-name');
  });
  await cookieStore.delete('cookie-name');
  const cookie = await cookieStore.get();
  assert_equals(cookie, null);
}, 'cookieStore.get returns null for a cookie deleted by cookieStore.delete');