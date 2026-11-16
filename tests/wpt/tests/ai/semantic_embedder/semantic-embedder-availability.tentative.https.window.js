// META: title=Embedder Availability
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const availability = await SemanticEmbedder.availability(embedderOptions());
  assert_in_array(availability, kValidAvailabilities);
}, 'SemanticEmbedder.availability() returns a valid value with an embeddingSpace');

promise_test(async t => {
  await promise_rejects_js(t, TypeError, SemanticEmbedder.availability());
  await promise_rejects_js(t, TypeError, SemanticEmbedder.availability({}));
  await promise_rejects_js(t, TypeError, SemanticEmbedder.availability(null));
  await promise_rejects_js(t, TypeError,
                           SemanticEmbedder.availability(undefined));
}, 'SemanticEmbedder.availability() requires an embeddingSpace');

promise_test(async () => {
  const availability = await SemanticEmbedder.availability(
      {embeddingSpace: 'unsupported-embedding-space'});
  assert_equals(availability, 'unavailable');
}, 'SemanticEmbedder.availability() is "unavailable" for an unknown embeddingSpace');
