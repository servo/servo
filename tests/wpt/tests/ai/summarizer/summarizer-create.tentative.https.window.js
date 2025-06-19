// META: title=Summarizer Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Summarizer);
  assert_equals(typeof Summarizer.create, 'function');
}, 'Summarizer.create() is defined');

promise_test(async t => {
  await testCreateMonitorCallbackThrowsError(t, createSummarizer);
}, 'If monitor throws an error, Summarizer.create() rejects with that error');