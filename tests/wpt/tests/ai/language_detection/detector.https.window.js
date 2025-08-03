// META: title=Detect english
// META: global=window
// META: timeout=long
// META: script=resources/util.js
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: script=../resources/locale-util.js

'use strict';

promise_test(async t => {
  // Creating the language detector without user activation rejects with
  // NotAllowedError.
  const createPromise = LanguageDetector.create();
  await promise_rejects_dom(t, 'NotAllowedError', createPromise);

  // Creating the translator with user activation succeeds.
  await createLanguageDetector();

  // Creating it should have switched it to available.
  const availability = await LanguageDetector.availability();
  assert_equals(availability, 'available');

  // Now that it is available, we should no longer need user activation.
  await LanguageDetector.create();
}, 'LanguageDetector.create() requires user activation when availability is "downloadable.');

promise_test(async t => {
  const detector = await createLanguageDetector();
  const results = await detector.detect(kTestPrompt);

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
  await testCreateMonitorCallbackThrowsError(t, createLanguageDetector);
}, 'If monitor throws an error, LanguageDetector.create() rejects with that error');

promise_test(async t => {
  testMonitor(createLanguageDetector);
}, 'LanguageDetector.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const createPromise = createLanguageDetector({signal: controller.signal});

  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'LanguageDetector.create() call with an aborted signal.');

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createLanguageDetector({signal});
  });
}, 'Aborting createLanguageDetector().');

promise_test(async t => {
  await testDestroy(t, createLanguageDetector, {}, [
    detector => detector.detect(kTestPrompt),
    detector => detector.measureInputUsage(kTestPrompt),
  ]);
}, 'Calling LanguageDetector.destroy() aborts calls to detect and measureInputUsage.');

promise_test(async t => {
  await testCreateAbort(t, createLanguageDetector, {}, [
    detector => detector.detect(kTestPrompt),
    detector => detector.measureInputUsage(kTestPrompt),
  ]);
}, 'LanguageDetector.create()\'s abort signal destroys its LanguageDetector after creation.');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const detector = await createLanguageDetector();
  const detectPromise =
    detector.detect(kTestPrompt, { signal: controller.signal });

  await promise_rejects_dom(t, 'AbortError', detectPromise);
}, 'LanguageDetector.detect() call with an aborted signal.');

promise_test(async t => {
  const detector = await createLanguageDetector();
  await testAbortPromise(t, signal => {
    return detector.detect(kTestPrompt, { signal });
  });
}, 'Aborting LanguageDetector.detect().');

promise_test(async t => {
  const detector = await createLanguageDetector();

  const text = 'Hello world!';
  const largeText = text.repeat(10000);
  const inputUsage = await detector.measureInputUsage(largeText);

  assert_greater_than_equal(detector.inputQuota, 0);
  assert_greater_than_equal(inputUsage, 0);

  const detectPromise = detector.detect(text);

  if (inputUsage >= detector.inputQuota) {
    await promise_rejects_quotaexceedederror(t, detectPromise, requested => requested !== null, detector.inputQuota);
  } else {
    await detectPromise;
  }
}, 'LanguageDetector.measureInputUsage() and inputQuota basic usage.');

promise_test(async t => {
  const controller = new AbortController();
  controller.abort();

  const detector = await createLanguageDetector();
  const measureInputUsagePromise =
    detector.measureInputUsage(kTestPrompt, { signal: controller.signal });

  await promise_rejects_dom(t, 'AbortError', measureInputUsagePromise);
}, 'LanguageDetector.measureInputUsage() call with an aborted signal.');

promise_test(async t => {
  const detector = await createLanguageDetector();
  await testAbortPromise(t, signal => {
    return detector.measureInputUsage(kTestPrompt, { signal });
  });
}, 'Aborting LanguageDetector.measureInputUsage().');

promise_test(async () => {
  const detector = await createLanguageDetector({expectedInputLanguages: []});
  assert_equals(detector.expectedInputLanguages, null);
}, 'Creating LanguageDetector with empty expectedInputLanguages array');

promise_test(async () => {
  const detector = await createLanguageDetector();
  assert_equals(detector.expectedInputLanguages, null);
}, 'Creating LanguageDetector without expectedInputLanguages');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, createLanguageDetector);
}, 'Progress events are not emitted after aborted.');
