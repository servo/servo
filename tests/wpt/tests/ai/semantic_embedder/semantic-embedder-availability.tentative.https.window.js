// META: title=Embedder Availability
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await SemanticEmbedder.availability();
  assert_in_array(availability, kValidAvailabilities);
}, 'SemanticEmbedder.availability() returns a valid value with no options');

promise_test(async () => {
  const availability = await SemanticEmbedder.availability({});
  assert_in_array(availability, kValidAvailabilities);
}, 'SemanticEmbedder.availability() returns a valid value with empty options');
