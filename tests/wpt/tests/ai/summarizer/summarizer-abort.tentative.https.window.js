// META: title=Summarizer Abort
// META: global=window,worker
// META: script=../resources/util.js

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return Summarizer.create({ signal: signal });
  });
}, "Aborting Summarizer.create().");

promise_test(async t => {
  const session = await Summarizer.create();
  await testAbortPromise(t, signal => {
    return session.summarize(kTestPrompt, { signal: signal });
  });
}, "Aborting Summarizer.summarize().");

promise_test(async t => {
  const session = await Summarizer.create();
  await testAbortReadableStream(t, signal => {
    return session.summarizeStreaming(kTestPrompt, { signal: signal });
  });
}, "Aborting Summarizer.summarizeStreaming().");
