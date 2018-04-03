
cookie_test(async t => {
  const theVeryRecentPast = Date.now();
  const expiredCookieSentinelValue = 'EXPIRED';
  await promise_rejects_when_unsecured(
    t,
    new TypeError(),
    cookieStore.set('__Secure-COOKIENAME', expiredCookieSentinelValue, {
      path: kPath,
      expires: theVeryRecentPast,
      secure: true,
      domain: location.hostname
    }),
    'Secure cookies only writable from secure contexts');

}, 'Set an already-expired secure cookie');

['__Host-', '__Secure-'].forEach(prefix => {
  cookie_test(async t => {
    const name = prefix + 'COOKIENAME';
    const value = 'cookie-value';

    await promise_rejects_when_unsecured(
      t,
      new TypeError(),
      cookieStore.set(name, value),
      `Setting ${prefix} cookies should fail in non-secure contexts`);

    // Getting does not produce an exception, even in non-secure contexts.
    const pair = await cookieStore.get(name);

    if (kIsUnsecured) {
      assert_equals(pair, null);
    } else {
      assert_equals(pair.value, value);
    }

    await promise_rejects_when_unsecured(
      t,
      new TypeError(),
      cookieStore.delete(name),
      `Deleting ${prefix} cookies should fail in non-secure contexts`);

    assert_equals(await cookieStore.get(name), null);
  }, `${prefix} cookies only writable from secure context`);
});
