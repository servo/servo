// META: title=Classifier Availability
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await Classifier.availability();
  // Assert that the result is a valid state.
  assert_in_array(availability, kValidAvailabilities);
}, 'Classifier.availability() returns a valid availability state');
