// META: title=Language Model Prompt Multimodal Image - SVGImageElement
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../../resources/util.js
// META: timeout=long

'use strict';

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

  // Must wait for the SVG and image to load first.
  const {promise, resolve} = Promise.withResolvers();
  svgImage.addEventListener('load', resolve);
  await promise;
  const session = await createLanguageModel(kImageOptions);
  const result =
      await session.prompt(messageWithContent(kImagePrompt, 'image', svgImage));
  assert_regexp_match(result, kValidImageRegex);
}, 'Prompt with SVGImageElement image content');
