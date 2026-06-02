// META: title=Language Model Prompt Multimodal Video
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  await ensureLanguageModel(kImageOptions);
  const blob = await (await fetch(kValidImagePath)).blob();
  const bitmap = await createImageBitmap(blob);
  const frame = new VideoFrame(bitmap, {timestamp: 1});
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', frame));
  frame.close();  // Avoid JS garbage collection warning.
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with VideoFrame image content');

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
      await session.prompt(messageWithContent(kImagePrompt, 'image', video));
  assert_regexp_match(result, kValidVideoRegex);
}, 'Prompt with HTMLVideoElement image content');
