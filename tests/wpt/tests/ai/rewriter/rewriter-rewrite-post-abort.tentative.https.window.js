// META: title=Rewriter Rewrite Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const rewriter = await createRewriter();
  const controller = new AbortController();
  const promise = rewriter.rewrite(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Rewrite again on the same session to ensure it is still usable.
  const result = await rewriter.rewrite(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Rewrite after aborting a previous rewrite.");
