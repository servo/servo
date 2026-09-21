// META: title=Summarizer Summarize Concurrent
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await createSummarizer();
  await Promise.all(
      [summarizer.summarize(kTestPrompt), summarizer.summarize(kTestPrompt)]);
}, 'Multiple Summarizer.summarize() calls are resolved successfully');
