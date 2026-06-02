// META: title=Summarizer Create Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await createSummarizer();
  assert_true(summarizer instanceof Summarizer);

  assert_equals(typeof summarizer.summarize, 'function');
  assert_equals(typeof summarizer.summarizeStreaming, 'function');
  assert_equals(typeof summarizer.measureInputUsage, 'function');
  assert_equals(typeof summarizer.destroy, 'function');

  assert_equals(typeof summarizer.expectedContextLanguages, 'object');
  assert_equals(typeof summarizer.expectedInputLanguages, 'object');
  assert_equals(typeof summarizer.inputQuota, 'number');
  assert_equals(typeof summarizer.outputLanguage, 'object');
  assert_equals(typeof summarizer.sharedContext, 'string');

  assert_equals(typeof summarizer.type, 'string');
  assert_equals(typeof summarizer.format, 'string');
  assert_equals(typeof summarizer.length, 'string');

  assert_equals(summarizer.type, 'key-points');
  assert_equals(summarizer.format, 'markdown');
  assert_equals(summarizer.length, 'short');
}, 'Summarizer.create() returns a valid object with default options');

promise_test(async () => {
  const summarizer = await testMonitor(createSummarizer);
  assert_equals(typeof summarizer, 'object');
}, 'Summarizer.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Summarizer.create);
}, 'Progress events are not emitted after aborted');

promise_test(async () => {
  const sharedContext = 'This is a shared context string';
  const summarizer = await createSummarizer({sharedContext: sharedContext});
  assert_equals(summarizer.sharedContext, sharedContext);
}, 'Summarizer.sharedContext');

promise_test(async () => {
  const summarizer = await createSummarizer({type: 'headline'});
  assert_equals(summarizer.type, 'headline');
}, 'Summarizer.type');

promise_test(async () => {
  const summarizer = await createSummarizer({format: 'plain-text'});
  assert_equals(summarizer.format, 'plain-text');
}, 'Summarizer.format');

promise_test(async () => {
  const summarizer = await createSummarizer({length: 'medium'});
  assert_equals(summarizer.length, 'medium');
}, 'Summarizer.length');

promise_test(async () => {
  const summarizer = await createSummarizer({expectedInputLanguages: ['en']});
  assert_array_equals(summarizer.expectedInputLanguages, ['en']);
}, 'Summarizer.expectedInputLanguages');

promise_test(async () => {
  const summarizer = await createSummarizer({expectedContextLanguages: ['en']});
  assert_array_equals(summarizer.expectedContextLanguages, ['en']);
}, 'Summarizer.expectedContextLanguages');

promise_test(async () => {
  const summarizer = await createSummarizer({outputLanguage: 'en'});
  assert_equals(summarizer.outputLanguage, 'en');
}, 'Summarizer.outputLanguage');

promise_test(async (t) => {
  return promise_rejects_js(
      t, RangeError,
      createSummarizer({expectedInputLanguages: ['en-abc-invalid']}));
}, 'Creating Summarizer with malformed language string');

promise_test(async (t) => {
  let summarizer = await createSummarizer({expectedInputLanguages: ['EN']});
  assert_true(!!summarizer);
}, 'Summarizer.create() canonicalizes language tags');

promise_test(async () => {
  const summarizer = await createSummarizer();
  assert_equals(summarizer.expectedInputLanguages, null);
  assert_equals(summarizer.expectedContextLanguages, null);
  assert_equals(summarizer.outputLanguage, null);
}, 'Summarizer optional attributes return null');
