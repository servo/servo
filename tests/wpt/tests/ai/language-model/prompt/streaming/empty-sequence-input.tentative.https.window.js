// META: title=Language Model Prompt Streaming - sequence empty
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();
  for await (const _ of model.promptStreaming([{ role: 'user', content: [] }])) { }
}, 'LanguageModel.promptStreaming() allows empty message sequence input');
