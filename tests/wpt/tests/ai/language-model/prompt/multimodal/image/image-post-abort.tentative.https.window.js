// META: title=Language Model Prompt Multimodal Image Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kImageOptions);

  const controller = new AbortController();
  const promise = session.prompt(
      messageWithContent(kImagePrompt, 'image', blob),
      { signal: controller.signal }
  );
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal image prompt after aborting a previous multimodal image prompt.");
