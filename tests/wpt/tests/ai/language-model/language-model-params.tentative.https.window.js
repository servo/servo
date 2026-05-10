// META: title=Language Model Params
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel();
  assert_true('params' in LanguageModel);
}, 'LanguageModel.params static accessor exists');

promise_test(async () => {
  await ensureLanguageModel();
  const params = await LanguageModel.params();
  assert_true(!!params);
  assert_equals(typeof params.defaultTopK, 'number');
  assert_equals(typeof params.maxTopK, 'number');
  assert_equals(typeof params.defaultTemperature, 'number');
  assert_equals(typeof params.maxTemperature, 'number');
}, 'LanguageModel.params() returns valid parameters');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  assert_equals(typeof session.topK, 'number');
  assert_equals(typeof session.temperature, 'number');
}, 'Default session has topK and temperature as numbers');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel({topK: 2, temperature: 0.6});
  assert_equals(session.topK, 2);
  assert_equals(session.temperature, Math.fround(0.6));
}, 'Create with topK and temperature returns a session with those values set');

promise_test(async () => {
  await ensureLanguageModel();
  const session = await createLanguageModel({topK: 5, temperature: 0.8});
  const clonedSession = await session.clone();
  assert_equals(clonedSession.topK, session.topK);
  assert_equals(clonedSession.temperature, session.temperature);
}, 'Clone preserves topK and temperature accessors');
