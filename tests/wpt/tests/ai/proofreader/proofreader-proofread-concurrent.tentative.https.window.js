// META: title=Proofreader Proofread Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const proofreader = await createProofreader();
  await Promise.all(
      [proofreader.proofread(kTestPrompt), proofreader.proofread(kTestPrompt)]);
}, 'Multiple Proofreader.proofread() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const proofreader = await createProofreader();
  await Promise.all([
    proofreader.proofread(kTestPrompt), proofreader.proofread(kTestPrompt2)
  ]);
}, 'Multiple Proofreader.proofread() calls with divergent inputs are resolved successfully');
