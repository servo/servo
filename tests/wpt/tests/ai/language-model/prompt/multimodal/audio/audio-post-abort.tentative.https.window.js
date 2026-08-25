// META: title=Language Model Prompt Multimodal Audio Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kAudioOptions);
  const blob = await (await fetch(kValidAudioPath)).blob();
  const session = await createLanguageModel(kAudioOptions);

  const controller = new AbortController();
  const promise = session.prompt(
      messageWithContent(kAudioPrompt, 'audio', blob),
      { signal: controller.signal }
  );
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal audio prompt after aborting a previous multimodal audio prompt.");
