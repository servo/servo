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

const legacyParamsEnabled = ('params' in LanguageModel);

if (legacyParamsEnabled) {
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
} else {
  promise_test(async t => {
    const session = await createLanguageModel({ samplingMode: 'balanced', temperature: 0.8 });
    assert_true(session !== null);
    assert_equals(session.samplingMode, 'balanced');
    assert_equals(session.temperature, undefined);
    session.destroy();
  }, 'LanguageModel.create() accepts a sampling mode and ignores unsupported temperature sampling option');

  promise_test(async t => {
    const session = await createLanguageModel({ samplingMode: 'balanced', topK: 10 });
    assert_true(session !== null);
    assert_equals(session.samplingMode, 'balanced');
    assert_equals(session.topK, undefined);
    session.destroy();
  }, 'LanguageModel.create() accepts a sampling mode and ignores unsupported topK sampling option');

  promise_test(async t => {
    const session = await createLanguageModel({ samplingMode: 'balanced', temperature: 0.8, topK: 10 });
    assert_true(session !== null);
    assert_equals(session.samplingMode, 'balanced');
    assert_equals(session.temperature, undefined);
    assert_equals(session.topK, undefined);
    session.destroy();
  }, 'LanguageModel.create() accepts a sampling mode and ignores unsupported temperature and topK sampling options');
}
