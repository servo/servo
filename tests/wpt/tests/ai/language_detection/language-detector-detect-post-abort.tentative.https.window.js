// META: title=LanguageDetector Detect Post-Abort
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const detector = await createLanguageDetector();
  const controller = new AbortController();
  const promise = detector.detect(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Detect again on the same session to ensure it is still usable.
  const result = await detector.detect(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Detect after aborting a previous detect.");
