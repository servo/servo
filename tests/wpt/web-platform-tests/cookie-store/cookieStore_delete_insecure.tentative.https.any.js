// META: title=Cookie Store API: cookieStore.delete() with insecure cookies
// META: global=!default,serviceworker,window

'use strict';

promise_test(async t => {
  await cookieStore.set('cookie-name', 'cookie-value', { secure: false });
  t.add_cleanup(async () => { await cookieStore.delete('cookie-name'); });

  await cookieStore.delete('cookie-name');
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete(name) can delete an insecure cookie');

promise_test(async t => {
  await cookieStore.set('cookie-name', 'cookie-value', { secure: false });
  t.add_cleanup(async () => { await cookieStore.delete('cookie-name'); });

  await cookieStore.delete({ name: 'cookie-name' });
  const cookie = await cookieStore.get('cookie-name');
  assert_equals(cookie, null);
}, 'cookieStore.delete(options) can delete an insecure cookie');