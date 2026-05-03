// META: title=Language Model Prompt Context Usage - Initial Prompt
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const options = {
    initialPrompts:
        [{role: 'system', content: 'The word of the day is banana.'}]
  };
  await ensureLanguageModel(options);
  const session = await createLanguageModel(options);
  const usage = await session.measureContextUsage(options.initialPrompts);
  assert_greater_than(usage, 0);
  assert_equals(session.contextUsage, usage);
  assert_regexp_match(
      await session.prompt('What is the word of the day?'), /banana/i);
}, 'Test that initialPrompt counts towards session contextUsage');
