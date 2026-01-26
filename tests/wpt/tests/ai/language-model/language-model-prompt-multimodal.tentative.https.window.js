// META: title=Language Model Prompt Multimodal
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

const kImagePrompt = 'describe this';
const kAudioPrompt = 'transcribe this';
const kValidImagePath = '/images/computer.jpg';
const kValidAudioPath = '/media/speech.wav';
const kValidSVGImagePath = '/images/pattern.svg';
const kValidVideoPath = '/media/test.webm';

const kImageOptions = {expectedInputs: [{type: 'image'}]};
const kAudioOptions = {expectedInputs: [{type: 'audio'}]};

function messageWithContent(prompt, type, value) {
  return [{
    role: 'user',
    content: [{type: 'text', value: prompt}, {type: type, value: value}]
  }];
}

/*****************************************
 * General tests
 *****************************************/

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

/*****************************************
 * Image tests
 *****************************************/

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
  assert_equals(session.inputUsage, tokenLength);
  assert_regexp_match(
      await session.prompt([{role: 'system', content: ''}]),
      /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Test Image initialPrompt');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', blob));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with Blob image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', bitmap));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with ImageBitmap image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const frame = new VideoFrame(bitmap, {timestamp: 1});
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', frame));
  frame.close();  // Avoid JS garbage collection warning.
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with VideoFrame image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const canvas = new OffscreenCanvas(512, 512);
  // Requires a context to convert to a bitmap.
  var context = canvas.getContext('2d');
  context.fillRect(10, 10, 200, 200);
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', canvas));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with OffscreenCanvas image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', new ImageData(256, 256)));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with ImageData image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const newImage = new Image();
  newImage.src = kValidImagePath;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', newImage));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with HTMLImageElement image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  var canvas = document.createElement('canvas');
  canvas.width = 1224;
  canvas.height = 768;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', canvas));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with HTMLCanvasElement image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(
      messageWithContent(kImagePrompt, 'image', await imageData.arrayBuffer()));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with ArrayBuffer image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const imageData = await fetch(kValidImagePath);
  const session = await createLanguageModel(kImageOptions);
  const result = await session.prompt(messageWithContent(
      kImagePrompt, 'image', new DataView(await imageData.arrayBuffer())));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
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

  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', imageView));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);

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
      await session.prompt(messageWithContent(
        kImagePrompt, 'image', newImage));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
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
      await session.prompt(messageWithContent(
        kImagePrompt, 'image', svgImage));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with SVGImageElement image content');

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  var video = document.createElement('video');
  video.src = kValidVideoPath;
  video.width = 1224;
  video.height = 768;
  // Make sure the video plays without requiring a gesture.
  video.muted = true;
  video.playsInline = true;
  video.autoplay = true;
  // Video must have frames fetched. See crbug.com/417249941#comment3
  await video.play();
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(
        kImagePrompt, 'image', video));
  assert_regexp_match(result, /image|picture|photo/i /* Expect the model to describe the file like "This {image, picture, photo}…". */);
}, 'Prompt with HTMLVideoElement image content');

/*****************************************
 * Audio tests
 *****************************************/

promise_test(async (t) => {
  await ensureLanguageModel();
  const blob = await (await fetch(kValidAudioPath)).blob();
  const session = await createLanguageModel();
  return promise_rejects_dom(
      t, 'NotSupportedError',
      session.prompt(messageWithContent(kImagePrompt, 'audio', blob)));
}, 'Prompt audio without `audio` expectedInput');

promise_test(async () => {
  const blob = await (await fetch(kValidAudioPath)).blob();
  const options = {
    expectedInputs: [{type: 'audio'}],
    initialPrompts: messageWithContent(kAudioPrompt, 'audio', blob)
  };
  await ensureLanguageModel(options);
  const session = await LanguageModel.create(options);
  const tokenLength = await session.measureInputUsage(options.initialPrompts);
  assert_greater_than(tokenLength, 0);
  assert_equals(session.inputUsage, tokenLength);
  assert_regexp_match(
      await session.prompt([{role: 'system', content: ''}]),
      /sentence/i /* Expect the model to transcribe the audio of "This is a sentence in a single segment". */);
}, 'Test Audio initialPrompt');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const blob = await (await fetch(kValidAudioPath)).blob();
  const session = await createLanguageModel(kAudioOptions);
  const result =
      await session.prompt(messageWithContent(kAudioPrompt, 'audio', blob));
  assert_regexp_match(result, /sentence/i /* Expect the model to transcribe the audio of "This is a sentence in a single segment". */);
}, 'Prompt with Blob audio content');

promise_test(async (t) => {
  await ensureLanguageModel(kAudioOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const session = await createLanguageModel(kAudioOptions);
  // TODO(crbug.com/409615288): Expect a TypeError according to the spec.
  return promise_rejects_dom(
      t, 'DataError',
      session.prompt(messageWithContent(kImagePrompt, 'audio', blob)));
}, 'Prompt audio with blob containing invalid audio data.');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const audio_data = await fetch(kValidAudioPath);
  const audioCtx = new AudioContext();
  const buffer = await audioCtx.decodeAudioData(await audio_data.arrayBuffer());
  const session = await createLanguageModel(kAudioOptions);
  const result =
      await session.prompt(messageWithContent(kAudioPrompt, 'audio', buffer));
  assert_regexp_match(result, /sentence/i /* Expect the model to transcribe the audio of "This is a sentence in a single segment". */);
}, 'Prompt with AudioBuffer');

promise_test(async () => {
  await ensureLanguageModel(kAudioOptions);
  const audio_data = await fetch(kValidAudioPath);
  const session = await createLanguageModel(kAudioOptions);
  const result = await session.prompt(
      messageWithContent(kAudioPrompt, 'audio', await audio_data.arrayBuffer()));
  assert_regexp_match(result, /sentence/i /* Expect the model to transcribe the audio of "This is a sentence in a single segment". */);
}, 'Prompt with BufferSource - ArrayBuffer');
