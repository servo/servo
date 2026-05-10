// META: title=Language Model Prompt Multimodal Image - ArrayBuffer
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', await imageData.arrayBuffer()));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ArrayBuffer image content');
