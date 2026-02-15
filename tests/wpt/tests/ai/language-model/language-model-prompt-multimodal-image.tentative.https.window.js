// META: title=Language Model Prompt Multimodal Image
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
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

promise_test(async () => {
  const blob = await (await fetch(kValidImagePath)).blob();
  const options = {
    expectedInputs: [{type: 'image'}],
    initialPrompts: messageWithContent(kImagePrompt, 'image', blob)
  };
  await ensureLanguageModel(options);
  const session = await LanguageModel.create(options);
  const tokenLength = await session.measureInputUsage(options.initialPrompts);
  assert_greater_than(tokenLength, 0);
  assert_true(isValueInRange(session.inputUsage, tokenLength));
  assert_regexp_match(
      await session.prompt([{role: 'system', content: ''}]), kValidImageRegex);
}, 'Test Image initialPrompt');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', blob));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with Blob image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', bitmap));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ImageBitmap image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const canvas = new OffscreenCanvas(512, 512);
  // Requires a context to convert to a bitmap.
  var context = canvas.getContext('2d');
  context.fillRect(10, 10, 200, 200);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', canvas));
  assert_regexp_match(result, kValidCanvasImageRegex);
}, 'Prompt with OffscreenCanvas image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', new ImageData(256, 256)));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ImageData image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidImagePath;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', newImage));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with HTMLImageElement image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  var canvas = document.createElement('canvas');
  canvas.width = 1224;
  canvas.height = 768;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', canvas));
  assert_regexp_match(result, kValidCanvasImageRegex);
}, 'Prompt with HTMLCanvasElement image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', await imageData.arrayBuffer()));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ArrayBuffer image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(messageWithContent(
      kImagePrompt, 'image', new DataView(await imageData.arrayBuffer())));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with ArrayBufferView image content');

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


promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidSVGImagePath;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', newImage));
  assert_regexp_match(result, kValidSVGImageRegex);
}, 'Prompt with HTMLImageElement image content (with SVG)');


promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('width', '100');
  svg.setAttribute('height', '100');
  const svgImage =
      document.createElementNS('http://www.w3.org/2000/svg', 'image');
  svgImage.setAttribute('href', kValidImagePath);
  svgImage.setAttribute('decoding', 'sync');
  svg.appendChild(svgImage);
  document.body.appendChild(svg);

  // Must wait for the SVG and image to load first.
  // TODO(crbug.com/417260923): Make prompt Api await the image to be loaded.
  const {promise, resolve} = Promise.withResolvers();
  svgImage.addEventListener('load', resolve);
  await promise;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', svgImage));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with SVGImageElement image content');
