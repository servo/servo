'use strict';

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await cookieStore.set('', 'first-value');
  const actual1 =
      (await cookieStore.getAll('')).map(({ value }) => value).join(';');
  const expected1 = 'first-value';
  assert_equals(actual1, expected1);
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: '', value: 'first-value'}]},
    'Observed no-name change');

  await promise_rejects(
    t,
    new TypeError(),
    cookieStore.set('', 'suspicious-value=resembles-name-and-value'),
    'Expected promise rejection when setting a cookie with' +
      ' no name and "=" in value');

  const actual2 =
        (await cookieStore.getAll('')).map(({ value }) => value).join(';');
  const expected2 = 'first-value';
  assert_equals(actual2, expected2);
  assert_equals(
    await getCookieString(),
    'first-value',
    'Earlier cookie jar after rejected');

  eventPromise = observeNextCookieChangeEvent();
  await cookieStore.delete('');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: '', value: ''}]},
    'Observed no-name deletion');

  assert_equals(
    await getCookieString(),
    undefined,
    'Empty cookie jar after cleanup');
  assert_equals(
    await getCookieStringHttp(),
    undefined,
    'Empty HTTP cookie jar after cleanup');

}, "Verify that attempting to set a cookie with no name and with '=' in" +
             " the value does not work.");
