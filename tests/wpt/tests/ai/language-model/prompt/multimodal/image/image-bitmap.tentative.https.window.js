// META: title=Language Model Prompt Multimodal Image - ImageBitmap
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', bitmap));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ImageBitmap image content');
