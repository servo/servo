// META: title=Writer Availability
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Writer);
  assert_equals(typeof Writer.availability, 'function');
}, 'Writer.availability() is defined');

promise_test(async () => {
  const availability = await Writer.availability();
  assert_in_array(availability, kValidAvailabilities);
}, 'Writer.availability() returns a valid value with no options');

promise_test(async () => {
  // An array of plausible test option values.
  const kCreateOptionsSpec = [
    {tone: [undefined, 'formal', 'neutral', 'casual']},
    {format: [undefined, 'plain-text', 'markdown']},
    {length: [undefined, 'short', 'medium', 'long']},
    {expectedInputLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {expectedContextLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {outputLanguage: [undefined, 'en', 'es', 'jp', 'fr']}
  ];
  for (const options of generateOptionCombinations(kCreateOptionsSpec)) {
    const availability = await Writer.availability(options);
    assert_in_array(availability, kValidAvailabilities, options);
  }
}, 'Writer.availability() returns a valid value with plausible options');
