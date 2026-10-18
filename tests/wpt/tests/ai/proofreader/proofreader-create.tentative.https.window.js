// META: title=Proofreader Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testCreateMonitorCallbackThrowsError(t, createProofreader);
}, 'If monitor throws an error, Proofreader.create() rejects with that error');
