// META: title=Classifier Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  await testAbortPromise(t, signal => {
    return createClassifier({ signal });
  });
}, 'Aborting Classifier.create().');

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');
  const classifier = await createClassifier();
  await testAbortPromise(t, signal => {
    return classifier.classify(kTestPrompt, { signal });
  });
}, 'Aborting Classifier.classify()');
