// META: title=Language Model Prompt Multimodal Image - ArrayBufferView Offset
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const buffer = await imageData.arrayBuffer();
  // Add 256 bytes of padding in front of the image data.
  const bufferView = new Uint8Array(buffer);
  const newBufferArray = new ArrayBuffer(256 + buffer.byteLength);
  const imageView = new Uint8Array(newBufferArray, 256, buffer.byteLength);
  imageView.set(bufferView);

  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', imageView));
  assert_regexp_match(result, kValidImageRegex);

  // Offset causes 56 bytes of blank data, resulting in a decoding error.
  await promise_rejects_dom(
      t, 'InvalidStateError',
      session.prompt(messageWithContent(
          kImagePrompt, 'image',
          new Uint8Array(newBufferArray, 200, buffer.byteLength))));
}, 'Prompt with ArrayBufferView image content with an offset.');
