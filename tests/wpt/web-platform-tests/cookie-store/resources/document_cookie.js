'use strict';

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await setCookieStringDocument('DOCUMENT-🍪=🔵; path=/');
  assert_equals(
      await getCookieString(),
      'DOCUMENT-🍪=🔵',
      'Cookie we wrote using document.cookie in cookie jar');
  assert_equals(
    await getCookieStringHttp(),
    'DOCUMENT-🍪=🔵',
    'Cookie we wrote using document.cookie in HTTP cookie jar');
  assert_equals(
      await getCookieStringDocument(),
      'DOCUMENT-🍪=🔵',
      'Cookie we wrote using document.cookie in document.cookie');
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: 'DOCUMENT-🍪', value: '🔵'}]},
      'Cookie we wrote using document.cookie is observed');

  eventPromise = observeNextCookieChangeEvent();
  await setCookieStringDocument('DOCUMENT-🍪=DELETED; path=/; max-age=0');
  assert_equals(
      await getCookieString(),
      undefined,
      'Empty cookie jar after document.cookie' +
        ' cookie-clearing using max-age=0');
  assert_equals(
    await getCookieStringHttp(),
    undefined,
    'Empty HTTP cookie jar after document.cookie' +
        ' cookie-clearing using max-age=0');
  assert_equals(
      await getCookieStringDocument(),
      undefined,
      'Empty document.cookie cookie jar after document.cookie' +
        ' cookie-clearing using max-age=0');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: 'DOCUMENT-🍪'}]},
      'Deletion observed after document.cookie cookie-clearing' +
        ' using max-age=0');
}, 'Verify interoperability of document.cookie with other APIs.');
