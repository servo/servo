// META: title=Language Model Params
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();

  assert_false(
      'params' in LanguageModel,
      'LanguageModel.params should not be defined on the LanguageModel interface in a window context.');
});
