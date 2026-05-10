// META: title=Classifier Create Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  assert_true(classifier instanceof Classifier);

  assert_equals(typeof classifier.classify, 'function');
  assert_equals(typeof classifier.destroy, 'function');
  assert_equals(typeof classifier.inputQuota, 'number');

}, 'Classifier.create() returns a valid object');

promise_test(async () => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await testMonitor(createClassifier);
  assert_equals(typeof classifier, 'object');
}, 'Classifier.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  await testCreateMonitorWithAbort(t, Classifier.create);
}, 'Progress events are not emitted after aborted');
