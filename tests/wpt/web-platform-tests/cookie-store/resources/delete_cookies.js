'use strict';

cookie_test(async testCase => {
  // TODO: This test doesn't create cookies and doesn't assert
  // the behavior of delete(). Improve or remove it.

  await cookieStore.delete('');
  await cookieStore.delete('TEST');
  await cookieStore.delete('META-🍪');
  await cookieStore.delete('DOCUMENT-🍪');
  await cookieStore.delete('HTTP-🍪');

  await setCookieStringHttp(
    'HTTPONLY-🍪=DELETED; path=/; max-age=0; httponly');

  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-COOKIENAME'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-1🍪'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-2🌟'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-3🌱'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-unordered1🍪'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-unordered2🌟'));
  await promise_rejects_when_unsecured(
      testCase,
      new TypeError(),
      cookieStore.delete('__Host-unordered3🌱'));
}, 'Test cookieStore.delete');
