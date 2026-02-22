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
  const measuredUsage = await session.measureContextUsage(initialPrompt);

  assert_greater_than(
      measuredUsage, contextWindow,
      'Measured usage should be greater than contextWindow');

  const promise = createLanguageModel(
      { initialPrompts: [ { role: "system", content: initialPrompt } ] });
  // Measured and actual usage may vary slightly for delimiter tokens.
  await promise_rejects_quotaexceedederror(t, promise, (actual) => {
    return isValueInRange(actual, measuredUsage);
  }, contextWindow);
}, 'QuotaExceededError is thrown when initial prompts are too large.');
