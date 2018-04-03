'use strict';

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await setCookieStringHttp('HTTP-🍪=🔵; path=/');
  assert_equals(
      await getCookieString(),
      'HTTP-🍪=🔵',
      'Cookie we wrote using HTTP in cookie jar');
  assert_equals(
      await getCookieStringHttp(),
      'HTTP-🍪=🔵',
      'Cookie we wrote using HTTP in HTTP cookie jar');
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: 'HTTP-🍪', value: '🔵'}]},
    'Cookie we wrote using HTTP is observed');

  eventPromise = observeNextCookieChangeEvent();
  await setCookieStringHttp('HTTP-🍪=DELETED; path=/; max-age=0');
  assert_equals(
      await getCookieString(),
      undefined,
      'Empty cookie jar after HTTP cookie-clearing using max-age=0');
  assert_equals(
      await getCookieStringHttp(),
      undefined,
      'Empty HTTP cookie jar after HTTP cookie-clearing using max-age=0');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: 'HTTP-🍪'}]},
    'Deletion observed after HTTP cookie-clearing using max-age=0');
  await cookieStore.delete('HTTP-🍪');
}, 'Interoperability of HTTP Set-Cookie: with other APIs');

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await setCookieStringHttp('HTTPONLY-🍪=🔵; path=/; httponly');
  assert_equals(
      await getCookieString(),
      undefined,
      'HttpOnly cookie we wrote using HTTP in cookie jar' +
        ' is invisible to script');
  assert_equals(
      await getCookieStringHttp(),
      'HTTPONLY-🍪=🔵',
    'HttpOnly cookie we wrote using HTTP in HTTP cookie jar');

  eventPromise = observeNextCookieChangeEvent();
  await setCookieStringHttp(
      'HTTPONLY-🍪=DELETED; path=/; max-age=0; httponly');
  assert_equals(
      await getCookieString(),
      undefined,
      'Empty cookie jar after HTTP cookie-clearing using max-age=0');
  assert_equals(
      await getCookieStringHttp(),
      undefined,
      'Empty HTTP cookie jar after HTTP cookie-clearing using max-age=0');

  // HTTPONLY cookie changes should not have been observed; perform
  // a dummy change to verify that nothing else was queued up.
  await cookieStore.set('TEST', 'dummy');
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: 'TEST', value: 'dummy'}]},
    'HttpOnly cookie deletion was not observed');
}, 'HttpOnly cookies are not observed');

cookie_test(async t => {
  // Non-UTF-8 byte sequences cause the Set-Cookie to be dropped.
  let eventPromise = observeNextCookieChangeEvent();
  await setCookieBinaryHttp(
      unescape(encodeURIComponent('HTTP-🍪=🔵')) + '\xef\xbf\xbd; path=/');
  assert_equals(
      await getCookieString(),
      'HTTP-🍪=🔵\ufffd',
      'Binary cookie we wrote using HTTP in cookie jar');
  assert_equals(
      await getCookieStringHttp(),
      'HTTP-🍪=🔵\ufffd',
      'Binary cookie we wrote using HTTP in HTTP cookie jar');
  assert_equals(
      decodeURIComponent(escape(await getCookieBinaryHttp())),
      'HTTP-🍪=🔵\ufffd',
      'Binary cookie we wrote in binary HTTP cookie jar');
  assert_equals(
      await getCookieBinaryHttp(),
      unescape(encodeURIComponent('HTTP-🍪=🔵')) + '\xef\xbf\xbd',
      'Binary cookie we wrote in binary HTTP cookie jar');
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: 'HTTP-🍪', value: '🔵\ufffd'}]},
    'Binary cookie we wrote using HTTP is observed');

  eventPromise = observeNextCookieChangeEvent();
  await setCookieBinaryHttp(
      unescape(encodeURIComponent('HTTP-🍪=DELETED; path=/; max-age=0')));
  assert_equals(
      await getCookieString(),
      undefined,
      'Empty cookie jar after binary HTTP cookie-clearing using max-age=0');
  assert_equals(
      await getCookieStringHttp(),
      undefined,
      'Empty HTTP cookie jar after' +
        ' binary HTTP cookie-clearing using max-age=0');
  assert_equals(
      await getCookieBinaryHttp(),
      undefined,
      'Empty binary HTTP cookie jar after' +
        ' binary HTTP cookie-clearing using max-age=0');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: 'HTTP-🍪'}]},
    'Deletion observed after binary HTTP cookie-clearing using max-age=0');
}, 'Binary HTTP cookies');
