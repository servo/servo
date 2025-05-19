// META: title=Summarizer Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createSummarizer({signal: signal});
  });
}, 'Aborting Summarizer.create().');

promise_test(async t => {
  const session = await createSummarizer();
  await testAbortPromise(t, signal => {
    return session.summarize(kTestPrompt, { signal: signal });
  });
}, 'Aborting Summarizer.summarize().');

promise_test(async t => {
  const session = await createSummarizer();
  await testAbortReadableStream(t, signal => {
    return session.summarizeStreaming(kTestPrompt, { signal: signal });
  });
}, 'Aborting Summarizer.summarizeStreaming().');
