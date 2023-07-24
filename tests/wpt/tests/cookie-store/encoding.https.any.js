// META: title=Cookie Store API: cookie encoding
// META: global=window,serviceworker
// META: script=resources/cookie-test-helpers.js

'use strict';

cookie_test(async t => {
  await setCookieStringHttp('\uFEFFcookie=value; path=/');
  const cookie = await cookieStore.get('\uFEFFcookie');
  assert_equals(cookie.name, '\uFEFFcookie');
  assert_equals(cookie.value, 'value');
}, 'BOM not stripped from name');

cookie_test(async t => {
  await setCookieStringHttp('cookie=\uFEFFvalue; path=/');
  const cookie = await cookieStore.get('cookie');
  assert_equals(cookie.name, 'cookie');
  assert_equals(cookie.value, '\uFEFFvalue');
}, 'BOM not stripped from value');
