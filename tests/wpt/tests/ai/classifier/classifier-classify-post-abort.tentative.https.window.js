// META: title=Classifier Classify Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const availability = await Classifier.availability();
  assert_implements_optional(availability !== 'unavailable', 'classifier is unavailable');

  const classifier = await createClassifier();
  const controller = new AbortController();
  const promise = classifier.classify(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Classify again on the same session to ensure it is still usable.
  const result = await classifier.classify(kTestPrompt);
  assert_equals(typeof result, 'string');
}, "Classify after aborting a previous classify.");
