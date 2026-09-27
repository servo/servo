// META: title=Summarizer Speed Preference measureInputUsage
// META: script=/resources/testdriver.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const summarizer = await createSummarizer({
    preference: 'speed',
    outputLanguage: 'en',
  });
  const result = await summarizer.measureInputUsage(kTestPrompt);
  assert_equals(typeof result, 'number');
  assert_greater_than(result, 0);
  assert_less_than(result, summarizer.inputQuota);
}, 'Summarizer.measureInputUsage() with speed preference returns non-empty result within inputQuota');
