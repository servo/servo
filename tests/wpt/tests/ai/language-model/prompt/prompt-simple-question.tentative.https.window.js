// META: title=Language Model Prompt - Prompt Simple Question
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const response = await session.prompt('What is the capital of France?');
  const isParis = /paris/i.test(response);
  const isEcho = /capital of france/i.test(response);
  assert_true(isParis || isEcho, "Response should be either an answer or an echo");
}, 'Check capital of France');
