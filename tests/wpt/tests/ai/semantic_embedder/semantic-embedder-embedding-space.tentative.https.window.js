// META: title=Embedder Embedding Space
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await promise_rejects_js(t, TypeError, SemanticEmbedder.create());
  await promise_rejects_js(t, TypeError, SemanticEmbedder.create({}));
  await promise_rejects_js(t, TypeError, SemanticEmbedder.create(null));
  await promise_rejects_js(t, TypeError, SemanticEmbedder.create(undefined));
}, 'SemanticEmbedder.create() requires an embeddingSpace');

promise_test(async t => {
  await promise_rejects_dom(
      t, 'NotSupportedError',
      SemanticEmbedder.create({embeddingSpace: 'unsupported-embedding-space'}));
}, 'SemanticEmbedder.create() rejects an unknown embeddingSpace');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  assert_equals(embedder.embeddingSpace, kEmbeddingSpace);
}, 'SemanticEmbedder.embeddingSpace echoes the space passed to create()');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result = await embedder.embed(['hello', 'world']);
  assert_equals(result.embeddings.length, 2);
  for (const embedding of result.embeddings) {
    assert_equals(embedding.embeddingSpace, kEmbeddingSpace);
  }
}, 'Each ContentEmbedding names the space its values are in');
