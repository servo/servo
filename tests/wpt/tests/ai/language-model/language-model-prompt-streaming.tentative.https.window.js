// META: title=Language Model Prompt Streaming
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const streamingResponse = session.promptStreaming(kTestPrompt);
  assert_true(streamingResponse instanceof ReadableStream);
  const result = (await Array.fromAsync(streamingResponse)).join('');
  assert_greater_than(result.length, 0, 'The result should not be empty.');
});

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();

  // null, undefined, and objects are coerced to strings.
  for await (const _ of model.promptStreaming(null)) { }
  for await (const _ of model.promptStreaming(undefined)) { }
  for await (const _ of model.promptStreaming({})) { }
  for await (const _ of model.promptStreaming('')) { }
  for await (const _ of model.promptStreaming([])) { }
  for await (const _ of model.promptStreaming([{ role: 'user', content: [] }])) { }
  for await (const _ of model.promptStreaming([{ role: 'user', content: [{ type: 'text', value: '' }] }])) { }
}, 'LanguageModel.promptStreaming() allows empty and coerced inputs');
