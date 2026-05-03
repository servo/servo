// META: title=Language Model - Sampling Mode Create
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const modes = ['most-predictable', 'predictable', 'balanced', 'creative', 'most-creative'];

  for (const mode of modes) {
    const session = await createLanguageModel({ samplingMode: mode });
    assert_true(session !== null, `Session created with ${mode} mode`);
    session.destroy();
  }
}, 'LanguageModel.create() accepts all valid sampling modes');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    createLanguageModel({ samplingMode: 'balanced', temperature: 0.8 })
  );
}, 'LanguageModel.create() rejects when both samplingMode and temperature are provided');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    createLanguageModel({ samplingMode: 'balanced', topK: 10 })
  );
}, 'LanguageModel.create() rejects when both samplingMode and topK are provided');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    createLanguageModel({ samplingMode: 'balanced', temperature: 0.8, topK: 10 })
  );
}, 'LanguageModel.create() rejects when samplingMode, temperature and topK are provided');
