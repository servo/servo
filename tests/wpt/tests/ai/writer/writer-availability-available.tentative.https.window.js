// META: title=Writer Availability Available
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Writer.availability();
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Writer.availability() is available with no options');

promise_test(async () => {
  const availability = await Writer.availability({
    tone: 'neutral',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['en-GB'],
    expectedContextLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Writer.availability() returns available with supported options');

promise_test(async t => {
  const options = {
    tone: 'neutral',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['zu'], // not supported
    expectedContextLanguages: ['en'],
    outputLanguage: 'zu', // not supported
  };
  const availability = await Writer.availability(options);
  assert_equals(availability, 'unavailable');
  await promise_rejects_dom(t, 'NotSupportedError', Writer.create(options));
}, 'Writer.availability() returns unavailable for unsupported languages and create() rejects');
