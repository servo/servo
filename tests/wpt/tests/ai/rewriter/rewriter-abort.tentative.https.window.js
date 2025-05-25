// META: title=Rewriter Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createRewriter({signal: signal});
  });
}, 'Aborting Rewriter.create()');

promise_test(async t => {
  const rewriter = await createRewriter();
  await testAbortPromise(t, signal => {
    return rewriter.rewrite(kTestPrompt, { signal: signal });
  });
}, 'Aborting Rewriter.rewrite()');

promise_test(async t => {
  const rewriter = await createRewriter();
  await testAbortReadableStream(t, signal => {
    return rewriter.rewriteStreaming(kTestPrompt, { signal: signal });
  });
}, 'Aborting Rewriter.rewriteStreaming()');

promise_test(async t => {
  const rewriter = await createRewriter();
  const controller = new AbortController();
  const streamingResponse = rewriter.rewriteStreaming(
      kTestPrompt, { signal: controller.signal });
  for await (const chunk of streamingResponse);  // Do nothing
  controller.abort();
}, 'Aborting Rewriter.rewriteStreaming() after finished reading');
