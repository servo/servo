// META: title=Language Model Prompt Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const session = await createLanguageModel();
  const controller = new AbortController();
  const promise = session.prompt(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Prompt after aborting a previous prompt.");
