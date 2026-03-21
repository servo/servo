// META: title=Summarizer Availability Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Summarizer.availability();
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() is available with no options');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: 'tldr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['en-GB'],
    expectedContextLanguages: ['en'],
    outputLanguage: 'en',
  });
  assert_in_array(availability, kAvailableAvailabilities);
}, 'Summarizer.availability() returns available with supported options');

promise_test(async t => {
  const options = {
    type: 'tldr',
    format: 'plain-text',
    length: 'medium',
    expectedInputLanguages: ['zu'], // not supported
    expectedContextLanguages: ['en'],
    outputLanguage: 'zu', // not supported
  };
  const availability = await Summarizer.availability(options);
  assert_equals(availability, 'unavailable');
  await promise_rejects_dom(t, 'NotSupportedError', Summarizer.create(options));
}, 'Summarizer.availability() returns unavailable for unsupported languages and create() rejects');
