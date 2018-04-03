'use strict';

// TODO(jsbell): Does this test really need simple origin ('__Host-') cookies?

cookie_test(async t => {
  await cookieStore.set('__Host-1🍪', '🔵cookie-value1🔴');
  await cookieStore.set('__Host-2🌟', '🌠cookie-value2🌠');
  await cookieStore.set('__Host-3🌱', '🔶cookie-value3🔷');
  // NOTE: this assumes no concurrent writes from elsewhere; it also
  // uses three separate cookie jar read operations where a single getAll
  // would be more efficient, but this way the CookieStore does the filtering
  // for us.
  const matchingValues = await Promise.all([ '1🍪', '2🌟', '3🌱' ].map(
      async suffix => (await cookieStore.get('__Host-' + suffix)).value));
  const actual = matchingValues.join(';');
  const expected = '🔵cookie-value1🔴;🌠cookie-value2🌠;🔶cookie-value3🔷';
  assert_equals(actual, expected);
}, 'Set three simple origin session cookies sequentially and ensure ' +
            'they all end up in the cookie jar in order.');

cookie_test(async t => {
  await Promise.all([
    cookieStore.set('__Host-unordered1🍪', '🔵unordered-cookie-value1🔴'),
    cookieStore.set('__Host-unordered2🌟', '🌠unordered-cookie-value2🌠'),
    cookieStore.set('__Host-unordered3🌱', '🔶unordered-cookie-value3🔷')
  ]);
  // NOTE: this assumes no concurrent writes from elsewhere; it also
  // uses three separate cookie jar read operations where a single getAll
  // would be more efficient, but this way the CookieStore does the filtering
  // for us and we do not need to sort.
  const matchingCookies = await Promise.all([ '1🍪', '2🌟', '3🌱' ].map(
    suffix => cookieStore.get('__Host-unordered' + suffix)));
  const actual = matchingCookies.map(({ value }) => value).join(';');
  const expected =
      '🔵unordered-cookie-value1🔴;' +
      '🌠unordered-cookie-value2🌠;' +
      '🔶unordered-cookie-value3🔷';
  assert_equals(actual, expected);
}, 'Set three simple origin session cookies in undefined order using ' +
            'Promise.all and ensure they all end up in the cookie jar in any ' +
            'order. ');
