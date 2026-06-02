// META: title=Language Model Prompt Multimodal
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidImagePath;
  const session = await createLanguageModel(kImageOptions);
  // TODO(crbug.com/409615288): Expect a TypeError according to the spec.
  return promise_rejects_dom(
      t, 'SyntaxError',
      session.prompt(messageWithContent(kImagePrompt, 'text', newImage)));
}, 'Prompt with type:"text" and image content should reject');

promise_test(async t => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidImagePath;
  const session = await createLanguageModel(kImageOptions);
  return promise_rejects_dom(t, 'NotSupportedError', session.prompt([
    {role: 'assistant', content: [{type: 'image', value: newImage}]}
  ]));
}, 'Prompt with assistant role should reject with multimodal input');
