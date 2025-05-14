// META: title=Summarizer measureInputUsage
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await Summarizer.create();
  const result = await summarizer.measureInputUsage(kTestPrompt);
  assert_equals(typeof result, 'number');
  assert_greater_than(result, 0);
}, 'Summarizer.measureInputUsage() returns non-empty result');
