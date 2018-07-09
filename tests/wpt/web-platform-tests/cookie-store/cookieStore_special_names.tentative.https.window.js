'use strict';

['__Secure-', '__Host-'].forEach(prefix => {
  promise_test(async testCase => {
    await cookieStore.set(`${prefix}cookie-name`, `secure-cookie-value`);
    assert_equals(
      (await cookieStore.get(`${prefix}cookie-name`)).value,
      'secure-cookie-value',
      `Setting ${prefix} cookies should not fail in secure context`);

    try { await cookieStore.delete(`${prefix}cookie-name`); } catch (e) {}
  }, `cookieStore.set with ${prefix} name on secure origin`);

  promise_test(async testCase => {
    // This test is for symmetry with the non-secure case. In non-secure
    // contexts, the set() should fail even if the expiration date makes
    // the operation a no-op.
    await cookieStore.set(
      `${prefix}cookie-name`, `secure-cookie-value`, {
        expires: Date.now() - (24 * 60 * 60 * 1000)
      });
    assert_equals(await cookieStore.get(`${prefix}cookie-name`), null);
    try { await cookieStore.delete(`${prefix}cookie-name`); } catch (e) {}
  }, `cookieStore.set of expired ${prefix} cookie name on secure origin`);

  promise_test(async testCase => {
    assert_equals(
      await cookieStore.delete(`${prefix}cookie-name`), undefined,
      `Deleting ${prefix} cookies should not fail in secure context`);
  }, `cookieStore.delete with ${prefix} name on secure origin`);
});
