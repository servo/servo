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
  const result = await rewriter.rewrite(kTestPrompt, {context: ' '});
  assert_not_equals(result, '');
}, 'Rewriter.rewrite() with a whitespace context returns a non-empty result');

promise_test(async t => {
  await testDestroy(t, createRewriter, {}, [
    rewriter => rewriter.rewrite(kTestPrompt),
    rewriter => rewriter.measureInputUsage(kTestPrompt),
  ]);
}, 'Calling Rewriter.destroy() aborts calls to rewrite and measureInputUsage.');

promise_test(async t => {
  await testCreateAbort(t, createRewriter, {}, [
    rewriter => rewriter.rewrite(kTestPrompt),
    rewriter => rewriter.measureInputUsage(kTestPrompt),
  ]);
}, 'Rewriter.create()\'s abort signal destroys its Rewriter after creation.');

promise_test(async () => {
  const rewriter = await createRewriter();
  const result = await rewriter.rewrite(kTestPrompt, {context: kTestContext});
  assert_equals(typeof result, 'string');
}, 'Simple Rewriter.rewrite() call');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all(
      [rewriter.rewrite(kTestPrompt), rewriter.rewrite(kTestPrompt)]);
}, 'Multiple Rewriter.rewrite() calls are resolved successfully');
