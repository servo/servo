// META: title=Summarizer Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createSummarizer({signal: signal});
  });
}, 'Aborting Summarizer.create().');

promise_test(async t => {
  const summarizer = await createSummarizer();
  await testAbortPromise(t, signal => {
    return summarizer.summarize(kTestPrompt, { signal: signal });
  });
}, 'Aborting Summarizer.summarize()');

promise_test(async t => {
  const summarizer = await createSummarizer();
  await testAbortReadableStream(t, signal => {
    return summarizer.summarizeStreaming(kTestPrompt, { signal: signal });
  });
}, 'Aborting Summarizer.summarizeStreaming()');

promise_test(async (t) => {
  const summarizer = await createSummarizer();
  const controller = new AbortController();
  const streamingResponse = summarizer.summarizeStreaming(
    kTestPrompt, { signal: controller.signal });
  for await (const chunk of streamingResponse);  // Do nothing
  controller.abort();
}, 'Aborting Summarizer.summarizeStreaming() after finished reading');

promise_test(async t => {
  const summarizer = await createSummarizer();
  await testAbortPromise(t, signal => {
    return summarizer.measureInputUsage(kTestPrompt, { signal });
  });
  const controller = new AbortController();
  controller.signal.addEventListener('abort', () => summarizer.destroy());
  const promise = summarizer.measureInputUsage(
      kTestPrompt, { signal: controller.signal });
  const err = new Error('test');
  controller.abort(err);
  await promise_rejects_exactly(t, err, promise);
}, 'Aborting Summarizer.measureInputUsage() and destroying on abort');
