// META: title=Language Model Prompt Multimodal Image - Without Image Expected Input
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const newImage = new Image();
  newImage.src = kValidImagePath;
  const session = await createLanguageModel();
  return promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(messageWithContent(kImagePrompt, 'image', newImage)));
}, 'Prompt image without `image` expectedInput');
