// META: title=Rewriter Availability Available
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Rewriter.availability();
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Rewriter.availability() is available with no options');

promise_test(async () => {
  const availability = await Rewriter.availability({
    tone: 'as-is',
    format: 'as-is',
    length: 'as-is',
    expectedInputLanguages: ['en-GB'],
    expectedContextLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Rewriter.availability() returns available with supported options');

promise_test(async () => {
  const availability = await Rewriter.availability({
    tone: 'as-is',
    format: 'as-is',
    length: 'as-is',
    expectedInputLanguages: ['zu'], // not supported
    expectedContextLanguages: ['en'],
    outputLanguage: 'zu', // not supported
  });
  assert_equals(availability, 'unavailable');
}, 'Rewriter.availability() returns unavailable for unsupported languages');
