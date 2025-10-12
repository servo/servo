// META: title=Rewriter Rewrite Streaming
// META: script=/common/gc.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const rewriter = await createRewriter();
  const streamingResponse =
    rewriter.rewriteStreaming(kTestPrompt, { context: kTestContext });
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    '[object ReadableStream]');
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_greater_than(result.length, 0);
}, 'Simple Rewriter.rewriteStreaming() call');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  const stream = rewriter.rewriteStreaming(kTestPrompt);

  rewriter.destroy();

  await promise_rejects_dom(
    t, 'AbortError', stream.pipeTo(new WritableStream()));
}, 'Rewriter.rewriteStreaming() fails after destroyed');

promise_test(async t => {
  const rewriter = await createRewriter();
  const streamingResponse = rewriter.rewriteStreaming('');
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  const { result, done } = await streamingResponse.getReader().read();
  assert_true(done);
}, 'Rewriter.rewriteStreaming() returns a ReadableStream without any chunk on an empty input');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all([
    rewriter.rewriteStreaming(kTestPrompt),
    rewriter.rewriteStreaming(kTestPrompt)
  ]);
}, 'Multiple Rewriter.rewriteStreaming() calls are resolved successfully');

promise_test(async () => {
  const rewriter = await createRewriter();
  const streamingResponse = rewriter.rewriteStreaming(kTestPrompt);
  garbageCollect();
  assert_equals(Object.prototype.toString.call(streamingResponse),
                '[object ReadableStream]');
  let result = '';
  for await (const value of streamingResponse) {
    result += value;
    garbageCollect();
  }
assert_greater_than(result.length, 0, 'The result should not be empty.');
}, 'Rewrite Streaming API must continue even after GC has been performed.');
