// META: title=Summarizer Availability
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Summarizer);
  assert_equals(typeof Summarizer.availability, 'function');
}, 'Summarizer.availability() is defined');

promise_test(async () => {
  const availability = await Summarizer.availability();
  assert_in_array(availability, kValidAvailabilities);
}, 'Summarizer.availability() returns a valid value with no options');

promise_test(async () => {
  // An array of plausible test option values.
  const kCreateOptionsSpec = [
    {type: [undefined, 'tldr', 'teaser', 'key-points', 'headline']},
    {format: [undefined, 'plain-text', 'markdown']},
    {length: [undefined, 'short', 'medium', 'long']},
    {expectedInputLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {expectedContextLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {outputLanguage: [undefined, 'en', 'es', 'jp', 'fr']}
  ];
  for (const options of generateOptionCombinations(kCreateOptionsSpec)) {
    const availability = await Summarizer.availability(options);
    assert_in_array(availability, kValidAvailabilities, options);
  }
}, 'Summarizer.availability() returns a valid value with plausible options');
