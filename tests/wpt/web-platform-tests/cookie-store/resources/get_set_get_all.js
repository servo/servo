'use strict';

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await cookieStore.set('TEST', 'value0');
  assert_equals(
      await getCookieString(),
      'TEST=value0',
      'Cookie jar contains only cookie we set');
  assert_equals(
    await getCookieStringHttp(),
    'TEST=value0',
    'HTTP cookie jar contains only cookie we set');
  await verifyCookieChangeEvent(
    eventPromise,
    {changed: [{name: 'TEST', value: 'value0'}]},
    'Observed value that was set');

  eventPromise = observeNextCookieChangeEvent();
  await cookieStore.set('TEST', 'value');
  assert_equals(
      await getCookieString(),
      'TEST=value',
      'Cookie jar contains only cookie we overwrote');
  await verifyCookieChangeEvent(
    eventPromise,
    {changed: [{name: 'TEST', value: 'value'}]},
    'Observed value that was overwritten');

  let allCookies = await cookieStore.getAll();
  assert_equals(
      allCookies[0].name,
      'TEST',
      'First entry in allCookies should be named TEST');
  assert_equals(
      allCookies[0].value,
      'value',
      'First entry in allCookies should have value "value"');
  assert_equals(
      allCookies.length,
      1,
      'Only one cookie should exist in allCookies');
  let firstCookie = await cookieStore.get();
  assert_equals(
      firstCookie.name,
      'TEST',
      'First cookie should be named TEST');
  assert_equals(
      firstCookie.value,
      'value',
      'First cookie should have value "value"');
  let allCookies_TEST = await cookieStore.getAll('TEST');
  assert_equals(
      allCookies_TEST[0].name,
      'TEST',
      'First entry in allCookies_TEST should be named TEST');
  assert_equals(
      allCookies_TEST[0].value,
      'value',
      'First entry in allCookies_TEST should have value "value"');
  assert_equals(
      allCookies_TEST.length,
      1,
      'Only one cookie should exist in allCookies_TEST');
  let firstCookie_TEST = await cookieStore.get('TEST');
  assert_equals(
      firstCookie_TEST.name,
      'TEST',
      'First TEST cookie should be named TEST');
  assert_equals(
      firstCookie_TEST.value,
      'value',
      'First TEST cookie should have value "value"');
}, 'Get/set/get all cookies in store');
