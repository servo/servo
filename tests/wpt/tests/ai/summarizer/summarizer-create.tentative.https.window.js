// META: title=Summarizer Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_implements_optional("Summarizer" in self, "Summarizer is not supported");
  assert_equals(typeof Summarizer.create, 'function');
}, 'Summarizer.create() is defined');

promise_test(async t => {
  await testCreateMonitorCallbackThrowsError(t, createSummarizer);
}, 'If monitor throws an error, Summarizer.create() rejects with that error');