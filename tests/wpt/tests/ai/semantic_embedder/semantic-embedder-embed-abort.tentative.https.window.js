// META: title=Embedder Embed Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  await testAbortPromise(t, signal => embedder.embed('hello', {signal}));
}, 'SemanticEmbedder.embed() can be aborted');

promise_test(async t => {
  await ensureEmbedder();
  const embedder = await createEmbedder();
  const controller = new AbortController();
  const result = await embedder.embed('hello', {signal: controller.signal});
  assert_equals(result.embeddings.length, 1);
  // Aborting after the promise settled does nothing.
  controller.abort();
  const again = await embedder.embed('hello');
  assert_equals(again.embeddings.length, 1);
}, 'Aborting after embed() resolved has no effect');
