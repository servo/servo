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
  const kCreateOptionsSpec = [
    {preference: ['speed']},
    {type: ['tldr', 'teaser', 'key-points', 'headline']},
    {format: ['plain-text']},
    {length: ['short', 'medium', 'long']},
    {expectedInputLanguages: [['en']]},
    {outputLanguage: ['en', 'es']},
  ];
  const combinations = generateOptionCombinations(kCreateOptionsSpec);
  let createdCount = 0;
  let rejectedCount = 0;
  for (const options of combinations) {
    try {
      const summarizer = await createSummarizer(options);
      assert_true(summarizer instanceof Summarizer);
      assert_equals(summarizer.type, options.type);
      assert_equals(summarizer.format, options.format);
      assert_equals(summarizer.length, options.length);
      assert_array_equals(
          summarizer.expectedInputLanguages, options.expectedInputLanguages);
      assert_equals(summarizer.outputLanguage, options.outputLanguage);
      summarizer.destroy();
      createdCount++;
    } catch (e) {
      assert_equals(e.name, 'NotSupportedError');
      rejectedCount++;
    }
  }
  assert_equals(createdCount + rejectedCount, combinations.length);
  assert_greater_than(createdCount, 0);
}, 'Summarizer.create() with speed preference handles option combinations');
