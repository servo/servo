// META: title=Cookie Store API: Observing 'change' events in document when modifications API is called with blank arguments
// META: script=resources/cookie-test-helpers.js

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

  eventPromise = observeNextCookieChangeEvent();
  await cookieStore.set('', '');
  const actual2 =
      (await cookieStore.getAll('')).map(({ value }) => value).join(';');
  const expected2 = '';
  assert_equals(actual2, expected2);
  await verifyCookieChangeEvent(
    eventPromise, {changed: [{name: '', value: ''}]},
    'Observed no-name change');

  eventPromise = observeNextCookieChangeEvent();
  await cookieStore.delete('');
  await verifyCookieChangeEvent(
    eventPromise, {deleted: [{name: ''}]},
    'Observed no-name deletion');

  assert_equals(
    await getCookieString(),
      undefined,
      'Empty cookie jar');
  assert_equals(
    await getCookieStringHttp(),
    undefined,
    'Empty HTTP cookie jar');
  if (kHasDocument) {
    assert_equals(
      await getCookieStringDocument(),
      undefined,
      'Empty document.cookie cookie jar');
  }

}, 'Verify behavior of no-name and no-value cookies.');
