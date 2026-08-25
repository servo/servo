// META: title=Writer measureInputUsage
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const writer = await createWriter();
  const result = await writer.measureInputUsage(kTestPrompt);
  assert_greater_than(result, 0);
}, 'Writer.measureInputUsage() returns non-empty result');
