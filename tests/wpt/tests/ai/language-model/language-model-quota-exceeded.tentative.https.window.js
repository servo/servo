// META: title=Language Model Quota Exceeded
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session to get the max tokens.
  const session = await createLanguageModel();
  const inputQuota = session.inputQuota;
  const initialPrompt = kTestPrompt.repeat(inputQuota);
  const requested = await session.measureInputUsage(initialPrompt);

  const promise = createLanguageModel(
      { initialPrompts: [ { role: "system", content: initialPrompt } ] });
  await promise_rejects_quotaexceedederror(t, promise, requested, inputQuota);
}, "QuotaExceededError is thrown when initial prompts are too large.");
