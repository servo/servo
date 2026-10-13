// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js

'use strict';

async function createSemanticEmbedder(options) {
  await test_driver.bless();
  return SemanticEmbedder.create(options);
}

promise_test(async t => {
  await testMonitor(createSemanticEmbedder);
}, 'SemanticEmbedder.create() notifies its monitor on downloadprogress');
