// META: title=Rewriter Availability
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Rewriter);
  assert_equals(typeof Rewriter.availability, 'function');
}, 'Rewriter.availability() is defined');

promise_test(async () => {
  const availability = await Rewriter.availability();
  assert_in_array(availability, kValidAvailabilities);
}, 'Rewriter.availability() returns a valid value with no options');

promise_test(async () => {
  // An array of plausible test option values.
  const kCreateOptionsSpec = [
    {tone: [undefined, 'as-is', 'more-formal', 'more-casual']},
    {format: [undefined, 'as-is', 'plain-text', 'markdown']},
    {length: [undefined, 'as-is', 'shorter', 'longer']},
    {expectedInputLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {expectedContextLanguages: [[], ['en'], ['es'], ['jp', 'fr']]},
    {outputLanguage: [undefined, 'en', 'es', 'jp', 'fr']}
  ];
  for (const options of generateOptionCombinations(kCreateOptionsSpec)) {
    const availability = await Rewriter.availability(options);
    assert_in_array(availability, kValidAvailabilities, options);
  }
}, 'Rewriter.availability() returns a valid value with plausible options');

promise_test(async (t) => {
  return promise_rejects_js(t, RangeError, Rewriter.availability({
    expectedInputLanguages: ['en-abc-invalid'],  // not supported
  }));
}, 'Rewriter.availability() rejects when given invalid language tags');
