// META: title=Language Model Prompt Streaming GC
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session.
  const session = await createLanguageModel();
  // Test the streaming prompt API.
  const streamingResponse =
    session.promptStreaming(kTestPrompt);
  // Run GC.
  gc();
  assert_equals(
    Object.prototype.toString.call(streamingResponse),
    "[object ReadableStream]"
  );
  let result = "";
  for await (const value of streamingResponse) {
    result += value;
    gc();
  }
  assert_greater_than(result.length, 0, "The result should not be empty.");
}, 'Prompt Streaming API must continue even after GC has been performed.');
