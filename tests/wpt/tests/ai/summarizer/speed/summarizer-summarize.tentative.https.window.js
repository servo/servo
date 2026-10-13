// META: title=Summarizer Speed Preference Summarize
// META: script=/resources/testdriver.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

const kSpeedOptions = {preference: 'speed', outputLanguage: 'en'};

promise_test(async () => {
  const summarizer = await createSummarizer(kSpeedOptions);
  let result = await summarizer.summarize('');
  assert_equals(result, '');
  result = await summarizer.summarize(' ');
  assert_equals(result, '');
}, 'Summarizer.summarize() with speed preference and empty input returns empty text');

promise_test(async () => {
  const summarizer = await createSummarizer(kSpeedOptions);
  const result = await summarizer.summarize(kTestPrompt);
  assert_equals(typeof result, 'string');
  assert_greater_than(result.length, 0);
}, 'Simple Summarizer.summarize() call with speed preference');

promise_test(async t => {
  await testDestroy(t, createSummarizer, kSpeedOptions, [
    summarizer => summarizer.summarize(kTestPrompt),
    summarizer => summarizer.measureInputUsage(kTestPrompt),
  ]);
}, 'Calling Summarizer.destroy() on speed preference session aborts calls to summarize and measureInputUsage');
