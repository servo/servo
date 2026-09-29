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
  const availability = await Summarizer.availability({
    preference: 'speed',
    type: 'tldr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['en-GB'],
    expectedContextLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() returns available with supported speed preference options');
