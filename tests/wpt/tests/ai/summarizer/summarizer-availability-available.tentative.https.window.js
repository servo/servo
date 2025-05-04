// META: title=Summarizer Availability Available
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Summarizer.availability();
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() is available with no options');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: 'tl;dr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['en-GB'],
    expectedContextLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() returns available with supported options');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: 'tl;dr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['es'], // not supported
    expectedContextLanguages: ['en'],
    outputLanguage: 'es', // not supported
  });
  assert_equals(availability, 'unavailable');
}, 'Summarizer.availability() returns unavailable for unsupported languages');
