// META: title=Summarizer Summarize Streaming Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await createSummarizer();
  const [result1, result2] = await Promise.all([
    readStream(summarizer.summarizeStreaming(kTestPrompt)),
    readStream(summarizer.summarizeStreaming(kTestPrompt)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Summarizer.summarizeStreaming() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const summarizer = await createSummarizer();
  const [result1, result2] = await Promise.all([
    readStream(summarizer.summarizeStreaming(kTestPrompt)),
    readStream(summarizer.summarizeStreaming(kTestPrompt2)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Summarizer.summarizeStreaming() calls with divergent inputs are resolved successfully');
