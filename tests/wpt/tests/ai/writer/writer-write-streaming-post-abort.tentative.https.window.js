// META: title=Writer Write Streaming Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const writer = await createWriter();
  const controller = new AbortController();
  const stream = writer.writeStreaming(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', stream.pipeTo(new WritableStream()));

  // Write again on the same session to ensure it is still usable.
  const streamingResponse = writer.writeStreaming(kTestPrompt);
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
}, "Write after aborting a previous writeStreaming.");
