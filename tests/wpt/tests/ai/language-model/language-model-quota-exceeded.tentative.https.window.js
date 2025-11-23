// META: title=Language Model Quota Exceeded
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// Helper function to check that 'actual' is within 'expected +/- delta'.
function isValueInRange(actual, expected, delta = 5) {
  const lowerBound = expected - delta;
  const upperBound = expected + delta;
  return actual >= lowerBound && actual <= upperBound;
}

promise_test(async t => {
  await ensureLanguageModel();

  // Start a new session to get the max tokens.
  const session = await createLanguageModel();
  const inputQuota = session.inputQuota;
  const initialPrompt = kTestPrompt.repeat(inputQuota);
  const measuredUsage = await session.measureInputUsage(initialPrompt);

  assert_greater_than(
      measuredUsage, inputQuota,
      'Measured usage should be greater than inputQuota');

  const promise = createLanguageModel(
      { initialPrompts: [ { role: "system", content: initialPrompt } ] });
  // Measured and actual usage may vary slightly for delimiter tokens.
  await promise_rejects_quotaexceedederror(t, promise, (actual) => {
    return isValueInRange(actual, measuredUsage);
  }, inputQuota);
}, 'QuotaExceededError is thrown when initial prompts are too large.');
