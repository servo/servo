// META: title=Language Model Prompt Multimodal Image - OffscreenCanvas
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const canvas = createColorGridCanvas(512, 512, /*isOffscreen=*/true);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', canvas));
  assert_regexp_match(result, kValidCanvasImageRegex);
}, 'Prompt with OffscreenCanvas image content');
