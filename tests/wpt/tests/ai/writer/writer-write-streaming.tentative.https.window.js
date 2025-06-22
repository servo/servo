// META: title=Writer Write Streaming
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const writer = await createWriter();
  const streamingResponse =
    writer.writeStreaming(kTestPrompt, { context: kTestContext });
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    '[object ReadableStream]');
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_greater_than(result.length, 0);
}, 'Simple Writer.writeStreaming() call');

promise_test(async (t) => {
  const writer = await createWriter();
  const stream = writer.writeStreaming(kTestPrompt);

  writer.destroy();

  await promise_rejects_dom(
    t, 'AbortError', stream.pipeTo(new WritableStream()));
}, 'Writer.writeStreaming() fails after destroyed');

promise_test(async t => {
  const writer = await createWriter();
  const streamingResponse = writer.writeStreaming('');
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  const { result, done } = await streamingResponse.getReader().read();
  assert_true(done);
}, 'Writer.writeStreaming() returns a ReadableStream without any chunk on an empty input');

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all([
    writer.writeStreaming(kTestPrompt),
    writer.writeStreaming(kTestPrompt)
  ]);
}, 'Multiple Writer.writeStreaming() calls are resolved successfully');

promise_test(async () => {
  const writer = await createWriter();
  const streamingResponse = writer.writeStreaming(kTestPrompt);
  gc();
  assert_equals(Object.prototype.toString.call(streamingResponse),
                '[object ReadableStream]');
  let result = '';
  for await (const value of streamingResponse) {
    result += value;
    gc();
  }
assert_greater_than(result.length, 0, 'The result should not be empty.');
}, 'Write Streaming API must continue even after GC has been performed.');
