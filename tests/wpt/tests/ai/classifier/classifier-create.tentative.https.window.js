// META: title=Classifier Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  assert_true(!!classifier, 'Classifier was successfully created');
}, 'Classifier.create() behavior depends on availability');

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  await testCreateMonitorCallbackThrowsError(t, createClassifier);
}, 'If monitor throws an error, Classifier.create() rejects with that error');
