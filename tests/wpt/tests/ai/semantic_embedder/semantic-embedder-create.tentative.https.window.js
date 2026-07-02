// META: title=Embedder Create
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureEmbedder();
  let embedder = await createEmbedder();
  assert_true(embedder instanceof SemanticEmbedder);

}, 'SemanticEmbedder.create() returns a valid object with default options');

promise_test(async t => {
  await ensureEmbedder();
  let embedder = await createEmbedder();
  let result = await embedder.embed("hello");
  assert_true(typeof result === 'object');
  assert_true(Array.isArray(result.embeddings));
  assert_equals(result.embeddings.length, 1);
  let embedding = result.embeddings[0];
  assert_true(embedding.values instanceof Float32Array);
  assert_true(embedding.values.length > 0);
  assert_equals(typeof embedding.values[0], 'number');
}, 'SemanticEmbedder.embed() returns SemanticEmbedderResult for single string');

promise_test(async t => {
  await ensureEmbedder();
  let embedder = await createEmbedder();
  let result = await embedder.embed(["hello", "world"]);
  assert_true(typeof result === 'object');
  assert_true(Array.isArray(result.embeddings));
  assert_equals(result.embeddings.length, 2);
  let embedding = result.embeddings[0];
  assert_true(embedding.values instanceof Float32Array);
  assert_true(embedding.values.length > 0);
  assert_equals(typeof embedding.values[0], 'number');
}, 'SemanticEmbedder.embed() returns SemanticEmbedderResult for batch strings');

promise_test(async t => {
  await ensureEmbedder();
  let embedder = await createEmbedder();
  let result = await embedder.embed("hello", { taskType: "classification" });
  assert_true(typeof result === 'object');
  assert_true(Array.isArray(result.embeddings));
  assert_equals(result.embeddings.length, 1);
}, 'SemanticEmbedder.embed() accepts taskType option');

promise_test(async t => {
  await ensureEmbedder();
  let embedder = await createEmbedder();
  let result = await embedder.embed([]);
  assert_true(typeof result === 'object');
  // It should gracefully resolve, potentially with undefined or empty embeddings.
  if (result.embeddings) {
    assert_equals(result.embeddings.length, 0);
  }
}, 'SemanticEmbedder.embed() handles empty array without crashing');

promise_test(async t => {
  await ensureEmbedder();
  await testAbortPromise(t, signal => createEmbedder({ signal }));
}, 'SemanticEmbedder.create() can be aborted');
