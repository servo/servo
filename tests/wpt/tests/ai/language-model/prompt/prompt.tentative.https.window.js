// META: title=Language Model Prompt
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const result = await session.prompt(kTestPrompt);
  assert_equals(typeof result, 'string');
}, 'Simple LanguageModel.prompt() call');


