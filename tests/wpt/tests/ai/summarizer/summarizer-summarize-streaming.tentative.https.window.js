// META: title=Summarizer Summarize Streaming
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const summarizer = await Summarizer.create();
  const streamingResponse = summarizer.summarizeStreaming(
    "The web-platform-tests Project is a cross-browser test suite for the Web-platform stack. Writing tests in a way that allows them to be run in all browsers gives browser projects confidence that they are shipping software that is compatible with other implementations, and that later implementations will be compatible with their implementations. This in turn gives Web authors/developers confidence that they can actually rely on the Web platform to deliver on the promise of working across browsers and devices without needing extra layers of abstraction to paper over the gaps left by specification editors and implementors.");
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  const reader = streamingResponse.getReader();
  let result = "";
  while (true) {
    const { value, done } = await reader.read();
    if (done) {
      break;
    }
    result = value;
  }
  assert_greater_than(result.length, 0);
}, 'Summarizer.summarizeStreaming() returns ReadableStream with a non-empty text.');

promise_test(async t => {
  const summarizer = await Summarizer.create();
  const streamingResponse = summarizer.summarizeStreaming("");
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  const { result, done } = await streamingResponse.getReader().read();
  assert_true(done);
}, 'Summarizer.summarizeStreaming() returns a ReadableStream without any chunk on an empty input.');

promise_test(async () => {
  const summarizer = await Summarizer.create();
  await Promise.all([
    summarizer.summarizeStreaming(kTestPrompt),
    summarizer.summarizeStreaming(kTestPrompt)
  ]);
}, 'Multiple Summarizer.summarizeStreaming() calls are resolved successfully.');
