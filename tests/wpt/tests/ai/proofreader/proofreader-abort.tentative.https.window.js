// META: title=Proofreader Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createProofreader({signal: signal});
  });
}, 'Aborting Proofreader.create()');

promise_test(async t => {
  const proofreader = await createProofreader();
  await testAbortPromise(t, signal => {
    return proofreader.proofread(kTestPrompt, { signal: signal });
  });
}, 'Aborting Proofreader.proofread()');

