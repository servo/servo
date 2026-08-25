// META: title=Rewriter Rewrite Streaming Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const rewriter = await createRewriter();
  const controller = new AbortController();
  const stream = rewriter.rewriteStreaming(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', stream.pipeTo(new WritableStream()));

  // Rewrite again on the same session to ensure it is still usable.
  const streamingResponse = rewriter.rewriteStreaming(kTestPrompt);
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
}, "Rewrite after aborting a previous rewriteStreaming.");
