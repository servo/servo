// META: title=Language Model Prompt Multimodal Image - ImageData
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const session = await createLanguageModel(kImageOptions);
  const canvas = createColorGridCanvas(256, 256);
  const context = canvas.getContext('2d');
  const imageData = context.getImageData(0, 0, 256, 256);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', imageData));
  assert_regexp_match(result, kValidCanvasImageRegex);
}, 'Prompt with ImageData image content');
