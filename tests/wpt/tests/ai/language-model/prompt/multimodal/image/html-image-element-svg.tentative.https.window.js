// META: title=Language Model Prompt Multimodal Image - HTMLImageElement SVG
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidSVGImagePath;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', newImage));
  assert_regexp_match(result, kValidSVGImageRegex);
}, 'Prompt with HTMLImageElement image content (with SVG)');
