// META: title=Language Model Prompt Streaming - array
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();
  for await (const _ of model.promptStreaming([])) { }
}, 'LanguageModel.promptStreaming() allows empty array input');
