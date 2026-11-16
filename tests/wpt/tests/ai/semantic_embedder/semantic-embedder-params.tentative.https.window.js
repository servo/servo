// META: title=Embedder Params
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  assert_true(!!SemanticEmbedder);
  const params = await SemanticEmbedder.params();
  assert_true(Array.isArray(params.supportedSpaces));
  // Yield PRECONDITION_FAILED if this browser serves no embedding space.
  assert_implements_optional(params.supportedSpaces.length > 0,
                             'No embedding space is supported');
  for (const space of params.supportedSpaces) {
    assert_equals(typeof space.embeddingSpace, 'string');
    assert_greater_than(space.embeddingSpace.length, 0);
    if (space.shutdownDate !== null) {
      assert_true(Number.isInteger(space.shutdownDate));
      assert_greater_than(space.shutdownDate, 0);
    }
  }
}, 'SemanticEmbedder.params() lists the served embedding spaces');

promise_test(async t => {
  assert_true(!!SemanticEmbedder);
  const params = await SemanticEmbedder.params();
  for (const space of params.supportedSpaces) {
    const availability = await SemanticEmbedder.availability(
        {embeddingSpace: space.embeddingSpace});
    assert_in_array(availability, kValidAvailabilities);
  }
}, 'availability() accepts every space listed by params()');
