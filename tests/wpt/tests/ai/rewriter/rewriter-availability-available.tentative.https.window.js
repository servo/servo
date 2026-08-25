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

promise_test(async t => {
  const options = {
    tone: 'as-is',
    format: 'as-is',
    length: 'as-is',
    expectedInputLanguages: ['zu'], // not supported
    expectedContextLanguages: ['en'],
    outputLanguage: 'zu', // not supported
  };
  const availability = await Rewriter.availability(options);
  assert_equals(availability, 'unavailable');
  await promise_rejects_dom(t, 'NotSupportedError', Rewriter.create(options));
}, 'Rewriter.availability() returns unavailable for unsupported languages and create() rejects');
