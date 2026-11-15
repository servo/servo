// META: title=Embedder Max Input Tokens
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  assert_equals(typeof embedder.maxInputTokens, 'number');
  assert_true(Number.isInteger(embedder.maxInputTokens));
  assert_greater_than(embedder.maxInputTokens, 0);
}, 'SemanticEmbedder.maxInputTokens is a positive integer');
