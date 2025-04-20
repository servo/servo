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


  // The last result should be 'und'.
  const undResult = results.pop();
  assert_equals(undResult.detectedLanguage, 'und');
  assert_greater_than(undResult.confidence, 0);

  let total_confidence_without_und = 0;
  let last_confidence = 1;
  for (const {confidence} of results) {
    assert_greater_than(confidence, 0);

    total_confidence_without_und += confidence;

    // Except for 'und', results should be from high to low confidence.
    assert_greater_than_equal(last_confidence, confidence);
    last_confidence = confidence;
  }

  // Confidences, excluding both 'und' and the last non-'und' result, should be
  // less than 0.99.
  assert_less_than(
      total_confidence_without_und - results.at(-1).confidence, 0.99);

  // Confidences, including 'und', should add up to 1.
  assert_equals(total_confidence_without_und + undResult.confidence, 1);
}, 'Simple LanguageDetector.detect() call');

promise_test(async t => {
  const error = new Error('CreateMonitorCallback threw an error');
  function monitor(m) {
    m.addEventListener('downloadprogress', e => {
      assert_unreached(
          'This should never be reached since monitor throws an error.');
    });
    throw error;
  }

  await promise_rejects_exactly(t, error, LanguageDetector.create({monitor}));
}, 'If monitor throws an error, LanguageDetector.create() rejects with that error');

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
  const detector = await LanguageDetector.create();

  const text = 'this string is in English';
  const promises = [detector.detect(text), detector.measureInputUsage(text)];

  detector.destroy();

  promises.push(detector.detect(text), detector.measureInputUsage(text));

  for (const promise of promises) {
    await promise_rejects_dom(t, 'AbortError', promise);
  }
}, 'Calling LanguageDetector.destroy() aborts calls to detect and measureInputUsage.');

promise_test(async t => {
  const controller = new AbortController();
  const detector = await LanguageDetector.create({signal: controller.signal});

  const text = 'this string is in English';
  const promises = [detector.detect(text), detector.measureInputUsage(text)];

  const error = new Error('The create abort signal was aborted.');
  controller.abort(error);

  promises.push(detector.detect(text), detector.measureInputUsage(text));

  for (const promise of promises) {
    await promise_rejects_exactly(t, error, promise);
  }
}, 'LanguageDetector.create()\'s abort signal destroys its LanguageDetector after creation.');

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
