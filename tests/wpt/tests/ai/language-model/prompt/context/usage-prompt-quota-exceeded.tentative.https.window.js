// META: title=Language Model Prompt Context Usage - Prompt Quota Exceeded
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptString = kTestPrompt.repeat(session.contextWindow);
  const usage = await session.measureContextUsage(promptString);
  try {
    await session.prompt(promptString);
    assert_unreached("Should have rejected with QuotaExceededError");
  } catch (e) {
    assert_equals(e.name, 'QuotaExceededError');
    // Allow some margin for counted tokens vs actual requested tokens.
    // TODO(crbug.com/500479741): Add expectation precision.
    assert_approx_equals(e.requested, usage, 50);
    assert_equals(e.quota, session.contextWindow);
  }
}, 'Test that prompt input exceeding the total context window rejects');
