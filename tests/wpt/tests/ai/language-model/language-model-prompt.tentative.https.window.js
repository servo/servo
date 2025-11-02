// META: title=Language Model Prompt
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const result = await session.prompt(kTestPrompt);
  assert_equals(typeof result, 'string');
}, 'Simple LanguageModel.prompt() call');

promise_test(async (t) => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_true(!!(await session.prompt([])));
  // Invalid input should be stringified.
  assert_regexp_match(await session.prompt({}), /\[object Object\]/);
}, 'Check empty input');

promise_test(async (t) => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_regexp_match(await session.prompt('shorthand'), /shorthand/);
  assert_regexp_match(
      await session.prompt([{role: 'system', content: 'shorthand'}]),
      /shorthand/);
}, 'Check Shorthand');

promise_test(async () => {
  const options = {
    initialPrompts:
        [{role: 'user', content: [{type: 'text', value: 'The word of the day is regurgitation.'}]}]
  };
  await ensureLanguageModel(options);
  const session = await LanguageModel.create(options);
  const tokenLength = await session.measureInputUsage(options.initialPrompts);
  assert_greater_than(tokenLength, 0);
  assert_equals(session.inputUsage, tokenLength);
  assert_regexp_match(
      await session.prompt([{role: 'system', content: ''}]),
      /regurgitation/);
}, 'Test that initialPrompt counts towards session inputUsage');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promise = new Promise(resolve => {
    session.addEventListener("quotaoverflow", resolve);
  });
  // Make sure there is something to evict.
  const kLongPrompt = kTestPrompt.repeat(10);
  const usage = await session.measureInputUsage(kLongPrompt);
  assert_greater_than(session.inputQuota, usage);
  await session.prompt(kLongPrompt);
  // Generate a repeated kLongPrompt string that exceeds inputQuota.
  assert_greater_than(session.inputUsage, 0);
  const repeatCount = session.inputQuota / session.inputUsage;
  const promptString = kLongPrompt.repeat(repeatCount);
  // The prompt promise succeeds, while causing older input to be evicted.
  await Promise.all([promise, session.prompt(promptString)]);
}, 'The `quotaoverflow` event is fired when overall usage exceeds the quota');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptString = kTestPrompt.repeat(session.inputQuota);
  const requested = await session.measureInputUsage(promptString);
  await promise_rejects_quotaexceedederror(t, session.prompt(promptString), requested, session.inputQuota);
}, 'Test that prompt input exeeding the total quota rejects');
