// META: title=Proofreader Proofread Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const proofreader = await createProofreader();
  const controller = new AbortController();
  const promise = proofreader.proofread(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Proofread again on the same session to ensure it is still usable.
  const result = await proofreader.proofread(kTestPrompt);
  assert_equals(typeof result, 'object');
  assert_not_equals(result.correctedInput, '');
}, "Proofread after aborting a previous proofread.");
