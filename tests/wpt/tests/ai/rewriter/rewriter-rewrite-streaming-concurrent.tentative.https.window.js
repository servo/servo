// META: title=Rewriter Rewrite Streaming Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const rewriter = await createRewriter();
  const [result1, result2] = await Promise.all([
    readStream(rewriter.rewriteStreaming(kTestPrompt)),
    readStream(rewriter.rewriteStreaming(kTestPrompt)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Rewriter.rewriteStreaming() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const rewriter = await createRewriter();
  const [result1, result2] = await Promise.all([
    readStream(rewriter.rewriteStreaming(kTestPrompt)),
    readStream(rewriter.rewriteStreaming(kTestPrompt2)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Rewriter.rewriteStreaming() calls with divergent inputs are resolved successfully');
