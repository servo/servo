// META: title=Rewriter Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Rewriter);
}, 'Rewriter must be defined.');

promise_test(async t => {
  await testCreateMonitorCallbackThrowsError(t, createRewriter);
}, 'If monitor throws an error, Rewriter.create() rejects with that error');
