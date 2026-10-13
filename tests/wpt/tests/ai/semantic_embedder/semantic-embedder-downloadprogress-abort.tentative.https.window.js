// META: title=Semantic Embedder Download Progress Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

async function createSemanticEmbedder(options) {
  await test_driver.bless();
  return SemanticEmbedder.create(options);
}

promise_test(async t => {
  await testCreateMonitorWithAbort(t, createSemanticEmbedder);
}, 'Progress events are not emitted after aborted.');
