// META: title=Writer Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Writer);
}, 'Writer must be defined.');

promise_test(async t => {
  await testCreateMonitorCallbackThrowsError(t, createWriter);
}, 'If monitor throws an error, Writer.create() rejects with that error');
