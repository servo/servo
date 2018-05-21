'use strict';

promise_test(async testCase => {
  const p = cookieStore.delete('cookie-name');
  assert_true(p instanceof Promise,
              'cookieStore.delete() returns a promise');
  const result = await p;
  assert_equals(result, undefined,
                'cookieStore.delete() promise resolves to undefined');
}, 'cookieStore.delete return type is Promise<void>');
