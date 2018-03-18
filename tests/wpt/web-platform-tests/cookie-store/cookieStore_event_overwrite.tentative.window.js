'use strict';

// Workaround because add_cleanup doesn't support async functions yet.
// See https://github.com/w3c/web-platform-tests/issues/6075
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

  await cookieStore.set('cookie-name', 'new-cookie-value');

  const event = await eventPromise;
  assert_true(event instanceof CookieChangeEvent);
  assert_equals(event.type, 'change');
  assert_equals(event.changed.length, 1);
  assert_equals(event.changed[0].name, 'cookie-name');
  assert_equals(event.changed[0].value, 'new-cookie-value');
  assert_equals(event.deleted.length, 0);

  await async_cleanup(() => cookieStore.delete('cookie-name'));
}, 'cookieStore fires change event for cookie overwritten by cookieStore.set()');
