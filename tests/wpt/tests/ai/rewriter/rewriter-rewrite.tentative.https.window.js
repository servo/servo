// META: title=Rewriter Rewrite
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const rewriter = await createRewriter();
  let result = await rewriter.rewrite('');
  assert_equals(result, '');
}, 'Rewriter.rewrite() with an empty input returns an empty text');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  let result = await rewriter.rewrite(' ');
  assert_equals(result, ' ');
}, 'Rewriter.rewrite() with a whitespace input returns a whitespace text');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  const result = await rewriter.rewrite(kTestPrompt, { context: ' ' });
  assert_not_equals(result, '');
}, 'Rewriter.rewrite() with a whitespace context returns a non-empty result');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  rewriter.destroy();
  await promise_rejects_dom(
    t, 'InvalidStateError', rewriter.rewrite(kTestPrompt));
}, 'Rewriter.rewrite() fails after destroyed');

promise_test(async () => {
  const rewriter = await createRewriter();
  const result = await rewriter.rewrite(kTestPrompt, { context: kTestContext });
  assert_equals(typeof result, 'string');
}, 'Simple Rewriter.rewrite() call');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all(
    [rewriter.rewrite(kTestPrompt), rewriter.rewrite(kTestPrompt)]);
}, 'Multiple Rewriter.rewrite() calls are resolved successfully');
