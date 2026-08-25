// META: title=Language Model Prompt Multimodal Video - VideoFrame
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const frame = new VideoFrame(bitmap, {timestamp: 1});
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', frame));
  frame.close();  // Avoid JS garbage collection warning.
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with VideoFrame image content');
