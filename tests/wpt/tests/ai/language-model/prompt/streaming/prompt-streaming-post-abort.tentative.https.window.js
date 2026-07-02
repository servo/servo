// META: title=Language Model Prompt Streaming Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const session = await createLanguageModel();
  const controller = new AbortController();
  const stream = session.promptStreaming(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', stream.pipeTo(new WritableStream()));

  // Prompt again on the same session to ensure it is still usable.
  const streamingResponse = session.promptStreaming(kTestPrompt);
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
}, "Prompt after aborting a previous promptStreaming.");
