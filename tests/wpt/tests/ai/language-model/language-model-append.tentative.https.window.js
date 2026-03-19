// META: title=Language Model Append
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const result = await session.append(kTestPrompt);
  assert_equals(result, undefined);
}, 'Simple LanguageModel.append() call');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_equals(session.contextUsage, 0);
  const promptUsage = await session.measureContextUsage(kTestPrompt);
  assert_greater_than(promptUsage, 0);
  await session.append(kTestPrompt);
  assert_equals(session.contextUsage, promptUsage);
}, 'Check contextUsage increases from a simple LanguageModel.append() call');

promise_test(async (t) => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_equals(session.contextUsage, 0);
  await session.append([]);
  assert_equals(session.contextUsage, 0);
  // Invalid input should be stringified.
  await session.append({});
  assert_greater_than(session.contextUsage, 0);
}, 'Check empty Object input for LanguageModel.append()');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptString = kTestPrompt.repeat(session.contextWindow);
  const usage = await session.measureContextUsage(promptString);
  await promise_rejects_quotaexceedederror(
      t, session.append(promptString), usage, session.contextWindow);
}, 'Test that append input exceeding the total context window rejects');
