// META: title=Summarizer Speed Preference Create Available
// META: script=/resources/testdriver.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await createSummarizer({
    preference: 'speed',
    outputLanguage: 'en',
  });
  assert_true(summarizer instanceof Summarizer);

  assert_equals(typeof summarizer.summarize, 'function');
  assert_equals(typeof summarizer.summarizeStreaming, 'function');
  assert_equals(typeof summarizer.measureInputUsage, 'function');
  assert_equals(typeof summarizer.destroy, 'function');

  assert_equals(typeof summarizer.inputQuota, 'number');
  assert_greater_than(summarizer.inputQuota, 0);

  assert_equals(summarizer.type, 'key-points');
  assert_equals(summarizer.format, 'markdown');
  assert_equals(summarizer.length, 'short');
}, 'Summarizer.create() returns a valid object with speed preference');

promise_test(async () => {
  const summarizer = await testMonitor(
      options => createSummarizer({
        preference: 'speed',
        outputLanguage: 'en',
        ...options,
      }));
  assert_equals(typeof summarizer, 'object');
}, 'Summarizer.create() with speed preference notifies its monitor on downloadprogress');

promise_test(async () => {
  const summarizer = await createSummarizer({
    preference: 'speed',
    type: 'tldr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_equals(summarizer.type, 'tldr');
  assert_equals(summarizer.format, 'plain-text');
  assert_equals(summarizer.length, 'medium');
  assert_array_equals(summarizer.expectedInputLanguages, ['en']);
  assert_equals(summarizer.outputLanguage, 'en');
}, 'Summarizer.create() with speed preference configures supported attributes');
