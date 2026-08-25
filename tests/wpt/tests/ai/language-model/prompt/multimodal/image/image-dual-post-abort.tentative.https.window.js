// META: title=Language Model Prompt Multimodal Image Dual Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kImageOptions);
  const imageBlob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kImageOptions);

  const message = [{
    role: 'user',
    content: [
      {type: 'image', value: imageBlob},
      {type: 'image', value: imageBlob},
      {type: 'text', value: 'compare these images'}
    ]
  }];

  const controller = new AbortController();
  const promise = session.prompt(message, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal prompt with dual image inputs after aborting a previous request.");
