// META: title=LanguageDetector Detect Concurrent
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const detector = await createLanguageDetector();
  await Promise.all(
      [detector.detect(kTestPrompt), detector.detect(kTestPrompt)]);
}, 'Multiple LanguageDetector.detect() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const detector = await createLanguageDetector();
  await Promise.all(
      [detector.detect(kTestPrompt), detector.detect(kTestPrompt2)]);
}, 'Multiple LanguageDetector.detect() calls with divergent inputs are resolved successfully');
