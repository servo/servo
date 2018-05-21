'use strict';

cookie_test(async t => {
  let eventPromise = observeNextCookieChangeEvent();
  await cookieStore.set('', 'first-value');
  assert_equals(
    (await cookieStore.getAll('')).map(({ value }) => value).join(';'),
    'first-value',
    'Cookie with no name and normal value should have been set');
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: '', value: 'first-value'}]},
    'Observed no-name change');

  await promise_rejects(
    t,
    new TypeError(),
    cookieStore.set('', 'suspicious-value=resembles-name-and-value'),
    'Expected promise rejection when setting a cookie with' +
      ' no name and "=" in value (via arguments)');

  await promise_rejects(
    t,
    new TypeError(),
    cookieStore.set(
      {name: '', value: 'suspicious-value=resembles-name-and-value'}),
    'Expected promise rejection when setting a cookie with' +
      ' no name and "=" in value (via options)');

  assert_equals(
    (await cookieStore.getAll('')).map(({ value }) => value).join(';'),
    'first-value',
    'Cookie with no name should still have previous value');

  eventPromise = observeNextCookieChangeEvent();
  await cookieStore.delete('');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: ''}]},
    'Observed no-name deletion');

}, "Verify that attempting to set a cookie with no name and with '=' in" +
             " the value does not work.");
