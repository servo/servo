// META: title=capabilities test

'use strict';

promise_test(async t => {
  const languageDetectorCapabilities = await ai.languageDetector.capabilities();
  const availability = languageDetectorCapabilities.available;
  assert_not_equals(availability, "no");
  // TODO(crbug.com/349927087): Add languageDetectorCapabilities.canDetect("en") once implemented.
});