// META: title=Summarizer Summarize Streaming
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const summarizer = await createSummarizer();
  const streamingResponse = summarizer.summarizeStreaming(kTestPrompt);
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_greater_than(result.length, 0);
}, 'Simple Summarizer.summarizeStreaming() call');

promise_test(async (t) => {
  const summarizer = await createSummarizer();
  const stream = summarizer.summarizeStreaming(kTestPrompt);

  summarizer.destroy();

  await promise_rejects_dom(
    t, 'AbortError', stream.pipeTo(new WritableStream()));
}, 'Summarizer.summarizeStreaming() fails after destroyed');

promise_test(async t => {
  const summarizer = await createSummarizer();
  const streamingResponse = summarizer.summarizeStreaming('');
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  const { result, done } = await streamingResponse.getReader().read();
  assert_true(done);
}, 'Summarizer.summarizeStreaming() returns a ReadableStream without any chunk on an empty input');

promise_test(async () => {
  const summarizer = await createSummarizer();
  await Promise.all([
    summarizer.summarizeStreaming(kTestPrompt),
    summarizer.summarizeStreaming(kTestPrompt)
  ]);
}, 'Multiple Summarizer.summarizeStreaming() calls are resolved successfully');

promise_test(async t => {
  const summarizer = await createSummarizer();
  const streamingResponse = summarizer.summarizeStreaming(kTestPrompt);
  gc();
  assert_equals(Object.prototype.toString.call(streamingResponse),
                '[object ReadableStream]');
  let result = '';
  for await (const value of streamingResponse) {
    result += value;
    gc();
  }
assert_greater_than(result.length, 0, 'The result should not be empty.');
}, 'Summarize Streaming API must continue even after GC has been performed.');
