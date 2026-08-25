// META: title=Language Model Prompt Empty Inputs - undefined
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();
  assert_regexp_match(await model.prompt(undefined), /undefined/);
}, 'LanguageModel.prompt() allows undefined input');
