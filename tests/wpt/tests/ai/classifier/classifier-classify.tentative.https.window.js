// META: title=Classifier Classify
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  let result = await classifier.classify('');
  assert_equals(typeof result, 'string');
  result = await classifier.classify(' ');
  assert_equals(typeof result, 'string');
}, 'Classifier.classify() with an empty input returns a string');

promise_test(async (t) => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  const result = await classifier.classify(kTestPrompt, {context: ' '});
  assert_equals(typeof result, 'string');
}, 'Classifier.classify() with a whitespace context returns a string');

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  await testDestroy(t, createClassifier, {}, [
    classifier => classifier.classify(kTestPrompt),
  ]);
}, 'Calling Classifier.destroy() aborts calls to classify.');

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  await testCreateAbort(t, createClassifier, {}, [
    classifier => classifier.classify(kTestPrompt),
  ]);
}, 'Classifier.create()\'s abort signal destroys its Classifier after creation.');

promise_test(async () => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  const result = await classifier.classify(kTestPrompt);
  assert_equals(typeof result, 'string');
}, 'Simple Classifier.classify() call');

promise_test(async () => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  await Promise.all(
      [classifier.classify(kTestPrompt), classifier.classify(kTestPrompt)]);
}, 'Multiple Classifier.classify() calls are resolved successfully');
