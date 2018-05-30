'use strict';

// Workaround because add_cleanup doesn't support async functions yet.
// See https://github.com/web-platform-tests/wpt/issues/6075
async function async_cleanup(cleanup_function) {
  try {
    await cleanup_function();
  } catch (e) {
    // Errors in cleanup functions shouldn't result in test failures.
  }
}

promise_test(async testCase => {
  await cookieStore.set('cookie-name', 'cookie-value');

  const eventPromise = new Promise((resolve) => {
    cookieStore.onchange = resolve;
  });
  await cookieStore.delete('cookie-name');
  const event = await eventPromise;
  assert_true(event instanceof CookieChangeEvent);
  assert_equals(event.type, 'change');
  assert_equals(event.deleted.length, 1);
  assert_equals(event.deleted[0].name, 'cookie-name');
  assert_equals(
      event.deleted[0].value, undefined,
      'Cookie change events for deletions should not have cookie values');
  assert_equals(event.changed.length, 0);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore fires change event for cookie deleted by cookieStore.delete()');