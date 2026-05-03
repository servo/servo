// META: title=Language Model Prompt Empty Inputs - sequence string
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();
  assert_equals(
      typeof await model.prompt(
          [{role: 'user', content: [{type: 'text', value: ''}]}]),
      'string');
}, 'LanguageModel.prompt() allows message sequence with empty string input');
