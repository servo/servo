// META: title=Embedder Input Items
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result = await embedder.embed({type: 'text', value: 'hello'});
  assert_equals(result.embeddings.length, 1);
  assert_true(result.embeddings[0].values instanceof Float32Array);
}, 'embed() accepts a {type: "text", value} item');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result =
      await embedder.embed(['hello', {type: 'text', value: 'world'}]);
  assert_equals(result.embeddings.length, 2);
}, 'embed() accepts a mixed sequence of strings and items');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError,
                           embedder.embed({type: 'image', value: 'x'}));
}, 'embed() rejects an item with an unknown type');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed({value: 'x'}));
}, 'embed() rejects an item without a type');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed({type: 'text'}));
}, 'embed() rejects an item without a value');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed(null));
}, 'embed(null) rejects instead of embedding the string "null"');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed(undefined));
}, 'embed(undefined) rejects instead of embedding the string "undefined"');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed({}));
}, 'embed({}) rejects instead of embedding "[object Object]"');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed([['a']]));
}, 'embed() rejects a nested array instead of embedding its string form');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await promise_rejects_js(t, TypeError, embedder.embed(['a', {}]));
}, 'embed() rejects a sequence with an object item that is not an input');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result = await embedder.embed(42);
  assert_equals(result.embeddings.length, 1);
}, 'embed() still stringifies a number');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result = await embedder.embed(true);
  assert_equals(result.embeddings.length, 1);
}, 'embed() still stringifies a boolean');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const result = await embedder.embed([]);
  assert_equals(result.embeddings.length, 0);
}, 'embed([]) resolves with no embeddings');
