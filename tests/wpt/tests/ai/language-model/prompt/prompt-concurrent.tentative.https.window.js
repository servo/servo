// META: title=Language Model Prompt Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  await Promise.all([session.prompt(kTestPrompt), session.prompt(kTestPrompt)]);
}, 'Multiple LanguageModel.prompt() calls with identical inputs are resolved successfully');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  await Promise.all(
      [session.prompt(kTestPrompt), session.prompt(kTestPrompt2)]);
}, 'Multiple LanguageModel.prompt() calls with divergent inputs are resolved successfully');
