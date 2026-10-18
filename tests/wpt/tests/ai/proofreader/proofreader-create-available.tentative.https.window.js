// META: title=Proofreader Create Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const proofreader = await createProofreader();
  assert_true(proofreader instanceof Proofreader);
  assert_equals(proofreader.includeCorrectionTypes, false);
  assert_equals(proofreader.includeCorrectionExplanations, false);
  assert_equals(proofreader.expectedInputLanguages, null);
  assert_equals(proofreader.correctionExplanationLanguage, null);
}, 'Proofreader.create() returns a valid object with default options');

promise_test(async () => {
  await testMonitor(createProofreader);
}, 'Proofreader.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Proofreader.create);
}, 'Proofreader.create() does not emit progress events after abort');

promise_test(async () => {
  const proofreader =
      await createProofreader({correctionExplanationLanguage: 'en'});
  assert_equals(proofreader.correctionExplanationLanguage, 'en');
}, 'Proofreader.create() sets correctionExplanationLanguage when specified');

promise_test(async () => {
  const proofreader = await createProofreader({expectedInputLanguages: ['en']});
  assert_array_equals(proofreader.expectedInputLanguages, ['en']);
}, 'Proofreader.create() sets expectedInputLanguages when specified');

promise_test(async () => {
  const proofreader = await createProofreader({includeCorrectionTypes: true});
  assert_equals(proofreader.includeCorrectionTypes, true);
}, 'Proofreader.create() sets includeCorrectionTypes when specified');

promise_test(async () => {
  const proofreader =
      await createProofreader({includeCorrectionExplanations: true});
  assert_equals(proofreader.includeCorrectionExplanations, true);
}, 'Proofreader.create() sets includeCorrectionExplanations when specified');

promise_test(async () => {
  const proofreader = await createProofreader({});
  assert_equals(proofreader.expectedInputLanguages, null);
  assert_equals(proofreader.correctionExplanationLanguage, null);
}, 'Proofreader.create() with empty options sets optional attributes to null');
