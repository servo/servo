// META: title=Language Model Quota Exceeded
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session to get the max tokens.
  const session = await createLanguageModel();
  const contextWindow = session.contextWindow;
  const initialPrompt = kTestPrompt.repeat(contextWindow);
  const usage = await session.measureContextUsage(initialPrompt);
  assert_greater_than(usage, contextWindow);
  const promise = createLanguageModel(
      { initialPrompts: [ { role: "system", content: initialPrompt } ] });
  await promise_rejects_quotaexceedederror(t, promise, usage, contextWindow);
}, 'QuotaExceededError is thrown when initial prompts are too large.');
