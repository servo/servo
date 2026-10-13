// META: title=Rewriter Rewrite Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all(
      [rewriter.rewrite(kTestPrompt), rewriter.rewrite(kTestPrompt)]);
}, 'Multiple Rewriter.rewrite() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all(
      [rewriter.rewrite(kTestPrompt), rewriter.rewrite(kTestPrompt2)]);
}, 'Multiple Rewriter.rewrite() calls with divergent inputs are resolved successfully');
