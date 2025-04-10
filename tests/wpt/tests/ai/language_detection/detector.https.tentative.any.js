// META: title=Detect english
// META: global=window,worker
// META: script=../resources/util.js

'use strict';

promise_test(async t => {
  // Language detection is available after call to `create()`.
  await LanguageDetector.create();
  const availability = await LanguageDetector.availability();
  assert_equals(availability, 'available');
}, 'Simple LanguageDetector.availability() call');

promise_test(async t => {
  const detector = await LanguageDetector.create();
  const results = await detector.detect('this string is in English');
  // "en" should be highest confidence.
  assert_equals(results[0].detectedLanguage, 'en');
  // Results should be from high to low confidence.
  for (let i = 0; i < results.length - 1; i++) {
    assert_greater_than_equal(results[i].confidence, results[i + 1].confidence);
  }
}, 'Simple LanguageDetector.detect() call');

promise_test(async t => {
  testMonitor(LanguageDetector.create);
}, 'LanguageDetector.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = LanguageDetector.create({signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'LanguageDetector.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return LanguageDetector.create({signal});
  });
}, 'Aborting LanguageDetector.create().');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const detector = await LanguageDetector.create();
  const detectPromise =
      detector.detect('this string is in English', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', detectPromise);
}, 'LanguageDetector.detect() call with an aborted signal.');

promise_test(async t => {
  const detector = await LanguageDetector.create();
  await testAbortPromise(t, signal => {
    return detector.detect('this string is in English', {signal});
  });
}, 'Aborting LanguageDetector.detect().');

promise_test(async t => {
  const detector = await LanguageDetector.create();

  const text = 'this string is in English';
  const inputUsage = await detector.measureInputUsage(text);

  assert_greater_than_equal(detector.inputQuota, 0);
  assert_greater_than_equal(inputUsage, 0);

  const detectPromise = detector.detect(text);

  if (inputUsage < detector.inputQuota) {
    assert_equals((await detectPromise)[0].detectedLanguage, 'en');
  } else {
    await promise_rejects_dom(t, 'QuotaExceededError', detectPromise);
  }
}, 'LanguageDetector.measureInputUsage() and inputQuota basic usage.');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const detector = await LanguageDetector.create();
  const measureInputUsagePromise =
      detector.measureInputUsage('hello', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', measureInputUsagePromise);
}, 'LanguageDetector.measureInputUsage() call with an aborted signal.');

promise_test(async t => {
  const detector = await LanguageDetector.create();
  await testAbortPromise(t, signal => {
    return detector.measureInputUsage('hello', {signal});
  });
}, 'Aborting LanguageDetector.measureInputUsage().');

promise_test(async () => {
  const expectedLanguages = ['en', 'es'];
  const detector = await LanguageDetector.create(
      {expectedInputLanguages: expectedLanguages});
  assert_array_equals(detector.expectedInputLanguages, expectedLanguages);
}, 'Creating LanguageDetector with expectedInputLanguages');
