// META: title=Scheduler: postTask Promise Value
// META: global=window,worker
'use strict';

promise_test(async t => {
  const result = await scheduler.postTask(() => 1234);
  assert_equals(result, 1234);
}, 'Test the task promise is resolved with the callback return value');
