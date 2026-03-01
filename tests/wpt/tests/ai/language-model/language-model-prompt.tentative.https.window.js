// META: title=Language Model Prompt
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
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
  assert_regexp_match(await session.prompt('What is the capital of France?'), /paris/i);
  assert_regexp_match(
      await session.prompt([{role: 'user', content: 'What is the capital of France?'}]),
      /paris/i);
}, 'Check capital of France');

promise_test(async () => {
  const options = {
    initialPrompts:
        [{role: 'system', content: [{type: 'text', value: 'The word of the day is regurgitation.'}]}]
  };
  await ensureLanguageModel(options);
  const session = await LanguageModel.create(options);
  const tokenLength = await session.measureContextUsage(options.initialPrompts);
  assert_greater_than(tokenLength, 0);
  assert_greater_than_equal(tokenLength, session.contextUsage);
  assert_regexp_match(
      await session.prompt([{role: 'user', content: 'What is the word of the day?'}]),
      /regurgitation/i);
}, 'Test that initialPrompt counts towards session contextUsage');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promise = new Promise(resolve => {
    session.addEventListener('contextoverflow', resolve);
  });
  // Make sure there is something to evict.
  const kLongPrompt = kTestPrompt.repeat(10);
  const usage = await session.measureContextUsage(kLongPrompt);
  assert_greater_than(session.contextWindow, usage);
  await session.prompt(kLongPrompt);
  // Generate a repeated kLongPrompt string that exceeds contextWindow.
  assert_greater_than(session.contextUsage, 0);
  const repeatCount = session.contextWindow / session.contextUsage;
  const promptString = kLongPrompt.repeat(repeatCount);
  // The prompt promise succeeds, while causing older input to be evicted.
  await Promise.all([promise, session.prompt(promptString)]);
}, 'The `contextoverflow` event is fired when overall usage exceeds the context window');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const promptString = kTestPrompt.repeat(session.contextWindow);
  const requested = await session.measureContextUsage(promptString);
  await promise_rejects_quotaexceedederror(
      t, session.prompt(promptString), requested, session.contextWindow);
}, 'Test that prompt input exceeding the total context window rejects');
