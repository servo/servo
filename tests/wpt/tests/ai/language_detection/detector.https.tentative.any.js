// META: title=Detect english
// META: global=window,worker

'use strict';

promise_test(async t => {
  const detector = await ai.languageDetector.create();
  const results = await detector.detect("this string is in English");
  // "en" should be highest confidence.
  assert_equals(results[0].detectedLanguage, "en");
  // Results should be from high to low confidence.
  for (let i = 0; i < results.length - 1; i++) {
    assert_greater_than_equal(results[i].confidence, results[i + 1].confidence);
  }
});
