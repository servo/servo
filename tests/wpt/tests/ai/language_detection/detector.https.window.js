// META: title=Detect english
// META: global=window
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

promise_test(async t => {
  // Language detection is available after call to `create()`.
  await LanguageDetector.create();
  const availability = await LanguageDetector.availability();
  assert_equals(availability, 'available');
}, 'Simple LanguageDetector.availability() call');

promise_test(async t => {
  const detector = await LanguageDetector.create();
  const results = await detector.detect('Hello world!');

  // must at least have the 'und' result.
  assert_greater_than_equal(results.length, 1);

  // The last result should be 'und'.
  const undResult = results.pop();
  assert_equals(undResult.detectedLanguage, 'und');
  assert_greater_than(undResult.confidence, 0);

  let total_confidence_without_und = 0;
  let last_confidence = 1;
  for (const {detectedLanguage, confidence} of results) {
    // All results must be in canonical form.
    assert_is_canonical(detectedLanguage);

    assert_greater_than(confidence, 0);
    assert_greater_than(confidence, undResult.confidence);

    total_confidence_without_und += confidence;

    // Except for 'und', results must be from high to low confidence.
    assert_greater_than_equal(last_confidence, confidence);
    last_confidence = confidence;
  }

  // If we have non-und results, their confidences, excluding the last non-'und'
  // result, must be less than 0.99.
  if (results.length > 0) {
    assert_less_than(
        total_confidence_without_und - results.at(-1).confidence, 0.99);
  }

  // Confidences, including 'und', should be less than or equal to one.
  assert_less_than_equal(
      total_confidence_without_und + undResult.confidence, 1);
}, 'LanguageDetector.detect() returns valid results');

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

  const text = 'Hello world!';
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

  const text = 'Hello world!';
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
      detector.detect('Hello world!', {signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', detectPromise);
}, 'LanguageDetector.detect() call with an aborted signal.');

promise_test(async t => {
  const detector = await LanguageDetector.create();
  await testAbortPromise(t, signal => {
    return detector.detect('Hello world!', {signal});
  });
}, 'Aborting LanguageDetector.detect().');

promise_test(async t => {
  const detector = await LanguageDetector.create();

  const text = 'Hello world!';
  const largeText = text.repeat(10000);
  const inputUsage = await detector.measureInputUsage(largeText);

  assert_greater_than_equal(detector.inputQuota, 0);
  assert_greater_than_equal(inputUsage, 0);

  const detectPromise = detector.detect(text);

  if (inputUsage >= detector.inputQuota) {
    await promise_rejects_dom(t, 'QuotaExceededError', detectPromise);
  } else {
    await detectPromise;
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
  const detector = await LanguageDetector.create();
  assert_equals(detector.expectedInputLanguages, null);
}, 'Creating LanguageDetector without expectedInputLanguages');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, LanguageDetector.create);
}, 'Progress events are not emitted after aborted.');
