// META: title=Language Model Prompt Multimodal Video Dual Post-Abort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap1 = await createImageBitmap(blob);
  const bitmap2 = await createImageBitmap(blob);
  const frame1 = new VideoFrame(bitmap1, {timestamp: 1});
  const frame2 = new VideoFrame(bitmap2, {timestamp: 2});
  const session = await createLanguageModel(kImageOptions);

  const message = [{
    role: 'user',
    content: [
      {type: 'image', value: frame1},
      {type: 'image', value: frame2},
      {type: 'text', value: 'compare these videos'}
    ]
  }];

  const controller = new AbortController();
  const promise = session.prompt(message, { signal: controller.signal });
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', promise);
  frame1.close();
  frame2.close();

  // Prompt again on the same session to ensure it is still usable.
  const result = await session.prompt(kTestPrompt);
  assert_greater_than(result.length, 0);
}, "Multimodal prompt with dual video inputs after aborting a previous request.");
