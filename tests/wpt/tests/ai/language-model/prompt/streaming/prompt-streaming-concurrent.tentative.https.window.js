// META: title=Language Model Prompt Streaming Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const [result1, result2] = await Promise.all([
    readStream(session.promptStreaming(kTestPrompt)),
    readStream(session.promptStreaming(kTestPrompt)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple LanguageModel.promptStreaming() calls with identical inputs are resolved successfully');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const [result1, result2] = await Promise.all([
    readStream(session.promptStreaming(kTestPrompt)),
    readStream(session.promptStreaming(kTestPrompt2)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple LanguageModel.promptStreaming() calls with divergent inputs are resolved successfully');
