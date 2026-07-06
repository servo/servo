// META: title=Language Model Prompt Multimodal Video - Without Video Expected Input
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const frame = new VideoFrame(bitmap, {timestamp: 1});
  const session = await createLanguageModel();
  const promise = session.prompt(messageWithContent(kImagePrompt, 'image', frame));
  await promise_rejects_dom(t, 'NotSupportedError', promise);
  frame.close();
}, 'Prompt video without `image` expectedInput');
