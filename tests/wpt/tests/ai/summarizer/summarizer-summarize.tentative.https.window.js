// META: title=Summarizer Summarize
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const summarizer = await createSummarizer();
  let result = await summarizer.summarize('');
  assert_equals(result, '');
  result = await summarizer.summarize(' ');
  assert_equals(result, '');
}, 'Summarizer.summarize() with an empty input returns an empty text');

promise_test(async (t) => {
  const summarizer = await createSummarizer();
  const result = await summarizer.summarize(kTestPrompt, { context: ' ' });
  assert_not_equals(result, '');
}, 'Summarizer.summarize() with a whitespace context returns an empty result');

promise_test(async (t) => {
  const summarizer = await createSummarizer();
  summarizer.destroy();
  await promise_rejects_dom(t, 'InvalidStateError', summarizer.summarize(kTestPrompt));
}, 'Summarizer.summarize() fails after destroyed');

promise_test(async () => {
  const summarizer = await createSummarizer();
  const result = await summarizer.summarize(kTestPrompt);
  assert_equals(typeof result, 'string');
  assert_greater_than(result.length, 0);
}, 'Simple Summarizer.summarize() call');

promise_test(async () => {
  const summarizer = await createSummarizer();
  await Promise.all([
    summarizer.summarize(kTestPrompt),
    summarizer.summarize(kTestPrompt)
  ]);
}, 'Multiple Summarizer.summarize() calls are resolved successfully');
