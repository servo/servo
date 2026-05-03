// META: title=Language Model Prompt Multimodal Image - Initial Prompt
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const blob = await (await fetch(kValidImagePath)).blob();
  const options = {
    expectedInputs: [{type: 'image'}],
    initialPrompts: messageWithContent(kImagePrompt, 'image', blob)
  };
  await ensureLanguageModel(options);
  const session = await createLanguageModel(options);
  const usage = await session.measureContextUsage(options.initialPrompts);
  assert_greater_than(usage, 0);
  assert_equals(session.contextUsage, usage);
  const result = await session.prompt('proceed');
  assert_regexp_match(result, kValidImageRegex);
}, 'Test Image initialPrompt');
