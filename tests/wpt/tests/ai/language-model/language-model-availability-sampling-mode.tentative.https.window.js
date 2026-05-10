// META: title=Language Model Prompt - Sampling Mode Availability
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const modes = ['most-predictable', 'predictable', 'balanced', 'creative', 'most-creative'];

  for (const mode of modes) {
    const result = await LanguageModel.availability({ samplingMode: mode });
    assert_true(kValidAvailabilities.includes(result),
      `Availability returned valid result for ${mode} mode`
    );
  }
}, 'LanguageModel.availability() accepts all valid sampling modes');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    LanguageModel.availability({ samplingMode: 'balanced', temperature: 0.8 })
  );
}, 'LanguageModel.availability() rejects when both samplingMode and temperature are provided');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    LanguageModel.availability({ samplingMode: 'balanced', topK: 10 })
  );
}, 'LanguageModel.availability() rejects when both samplingMode and topK are provided');

promise_test(async t => {
  await promise_rejects_js(
    t,
    TypeError,
    LanguageModel.availability({ samplingMode: 'balanced', temperature: 0.8, topK: 10 })
  );
}, 'LanguageModel.availability() rejects when samplingMode, temperature and topK are provided');
