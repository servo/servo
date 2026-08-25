// META: title=Language Model Prompt Context Usage
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_equals(session.contextUsage, 0);
  const promptUsage = await session.measureContextUsage(kTestPrompt);
  assert_greater_than(promptUsage, 0);
  const result = await session.prompt(kTestPrompt);
  assert_equals(typeof result, 'string');
  const resultUsage = await session.measureContextUsage(result);
  // The response is also appended to the session context, increasing usage.
  // Allow some margin for counted tokens vs actual requested tokens.
  // TODO(crbug.com/500479741): Add expectation precision.
  assert_approx_equals(session.contextUsage, promptUsage + resultUsage, 5);
}, 'Check contextUsage increases from a simple LanguageModel.prompt() call');
