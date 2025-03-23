// META: title=Detect english
// META: global=window,worker
// META: script=../resources/util.js

'use strict';

promise_test(async t => {
  // Language detection is available after call to `create()`.
  await ai.languageDetector.create();
  const availability = await ai.languageDetector.availability();
  assert_equals(availability, 'available');
}, 'Simple AILanguageDetector.availability() call');

promise_test(async t => {
  const detector = await ai.languageDetector.create();
  const results = await detector.detect('this string is in English');
  // "en" should be highest confidence.
  assert_equals(results[0].detectedLanguage, 'en');
  // Results should be from high to low confidence.
  for (let i = 0; i < results.length - 1; i++) {
    assert_greater_than_equal(results[i].confidence, results[i + 1].confidence);
  }
}, 'Simple AILanguageDetector.detect() call');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = ai.languageDetector.create({signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'AILanguageDetectorFactory.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return ai.languageDetector.create({signal});
  });
}, 'Aborting AILanguageDetectorFactory.create().');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const detector = await ai.languageDetector.create();
  const detectPromise =
      detector.detect('this string is in English', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', detectPromise);
}, 'AILanguageDetector.detect() call with an aborted signal.');

promise_test(async t => {
  const detector = await ai.languageDetector.create();
  await testAbortPromise(t, signal => {
    return detector.detect('this string is in English', {signal});
  });
}, 'Aborting AILanguageDetector.detect().');
