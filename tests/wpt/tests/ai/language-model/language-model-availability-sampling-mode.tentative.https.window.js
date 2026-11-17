// META: title=Language Model Prompt - Sampling Mode Availability
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const modes = [
    'most-predictable',
    'predictable',
    'slightly-predictable',
    'balanced',
    'slightly-creative',
    'creative',
    'most-creative',
  ];

  for (const mode of modes) {
    const result = await LanguageModel.availability({ samplingMode: mode });
    assert_true(kValidAvailabilities.includes(result),
      `Availability returned valid result for ${mode} mode`
    );
  }
}, 'LanguageModel.availability() accepts all valid sampling modes');

promise_test(async t => {
  const result = await LanguageModel.availability(
      {samplingMode: 'most-predictable', temperature: 0.8});
  assert_true(kValidAvailabilities.includes(result));
}, 'LanguageModel.availability() accepts a sampling mode and ignores legacy temperature sampling option');

promise_test(async t => {
  const result =
      await LanguageModel.availability({samplingMode: 'predictable', topK: 10});
  assert_true(kValidAvailabilities.includes(result));
}, 'LanguageModel.availability() accepts a sampling mode and ignores legacy topK sampling option');

promise_test(async t => {
  const result = await LanguageModel.availability(
      {samplingMode: 'creative', temperature: 0.8, topK: 10});
  assert_true(kValidAvailabilities.includes(result));
}, 'LanguageModel.availability() accepts a sampling mode and ignores legacy sampling options');

promise_test(async t => {
  const invalidModes = ['invalid', '', 'Balanced', null, 123];
  for (const mode of invalidModes) {
    await promise_rejects_js(
        t, TypeError, LanguageModel.availability({samplingMode: mode}),
        `availability() should reject with TypeError for samplingMode=${
            JSON.stringify(mode)}`);
  }
  await promise_rejects_js(
      t, TypeError,
      LanguageModel.availability(
          {samplingMode: 'invalid', temperature: 0.8, topK: 10}),
      'availability() should reject invalid samplingMode even with legacy options');
}, 'LanguageModel.availability() rejects with TypeError for invalid sampling modes');
