// META: title=Language Model Prompt Streaming
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
    if (value) {
      result += value;
    }
  }
  assert_greater_than(result.length, 0, "The result should not be empty.");
});
