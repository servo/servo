// META: title=canDetect

'use strict';

promise_test(async t => {
  const canDetect = await translation.canDetect();
  assert_greater_than(canDetect.length, 0);
  assert_not_equals(canDetect, "no");
});
