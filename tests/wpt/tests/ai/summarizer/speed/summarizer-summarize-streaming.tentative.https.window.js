// META: title=Summarizer Speed Preference Summarize Streaming
// META: script=/resources/testdriver.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

const kSpeedOptions = {preference: 'speed', outputLanguage: 'en'};

promise_test(async () => {
  const summarizer = await createSummarizer(kSpeedOptions);
  const streamingResponse = summarizer.summarizeStreaming(kTestPrompt);
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
}, 'Simple Summarizer.summarizeStreaming() call with speed preference');

promise_test(async t => {
  const summarizer = await createSummarizer(kSpeedOptions);
  const stream = summarizer.summarizeStreaming(kTestPrompt);

  summarizer.destroy();

  await promise_rejects_dom(
      t, 'AbortError', stream.pipeTo(new WritableStream()));
}, 'Summarizer.summarizeStreaming() with speed preference fails after destroyed');

promise_test(async () => {
  const summarizer = await createSummarizer(kSpeedOptions);
  const streamingResponse = summarizer.summarizeStreaming('');
  assert_true(streamingResponse instanceof ReadableStream);
  const {done} = await streamingResponse.getReader().read();
  assert_true(done);
}, 'Summarizer.summarizeStreaming() with speed preference returns a ReadableStream without any chunk on an empty input');
