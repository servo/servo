// META: title=Summarizer Create Available
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await Summarizer.create();
  const result = await summarizer.summarize(kTestPrompt);
  assert_equals(typeof result, 'string');
  assert_greater_than(result.length, 0);
}, 'Summarizer.summarize() returns non-empty result.');

promise_test(async () => {
  const summarizer = await Summarizer.create();
  await Promise.all([
    summarizer.summarize(kTestPrompt),
    summarizer.summarize(kTestPrompt)
  ]);
}, 'Multiple Summarizer.summarize() calls are resolved successfully.');
