// META: title=Summarizer Speed Preference Availability Available
// META: script=/resources/testdriver.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Summarizer.availability({
    preference: 'speed',
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() is available with speed preference');

promise_test(async () => {
  const kCreateOptionsSpec = [
    {preference: ['speed']},
    {type: [undefined, 'tldr', 'teaser', 'key-points', 'headline']},
    {format: [undefined, 'plain-text', 'markdown']},
    {length: [undefined, 'short', 'medium', 'long']},
    {expectedInputLanguages: [undefined, ['en-GB'], ['es']]},
    {expectedContextLanguages: [undefined, ['en']]},
    {outputLanguage: [undefined, 'en', 'es']},
  ];
  let availableCount = 0;
  for (const options of generateOptionCombinations(kCreateOptionsSpec)) {
    const availability = await Summarizer.availability(options);
    assert_in_array(
        availability, kValidAvailabilities, JSON.stringify(options));
    if (kAvailableAvailabilities.includes(availability)) {
      availableCount++;
    }
  }
  assert_greater_than(availableCount, 0);
}, 'Summarizer.availability() returns valid availability across speed preference options');
