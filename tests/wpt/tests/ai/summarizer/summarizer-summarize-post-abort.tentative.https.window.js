// META: title=Summarizer Summarize Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  const summarizer = await createSummarizer();
  const controller = new AbortController();
  const promise = summarizer.summarize(kTestPrompt, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Summarize again on the same session to ensure it is still usable.
  const result = await summarizer.summarize(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Summarize after aborting a previous summarize.");
