// META: title=Writer Write Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const writer = await createWriter();
  const controller = new AbortController();
  const promise = writer.write(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Write again on the same session to ensure it is still usable.
  const result = await writer.write(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Write after aborting a previous write.");
