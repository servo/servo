// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=resources/picture-in-picture-helpers.js

'use strict';

// https://wicg.github.io/picture-in-picture/

promise_test(async () => {
  try {
    self.video = await loadVideo();
    self.pipw = await requestPictureInPictureWithTrustedClick(video);
  } catch (e) {
    // Will be surfaced when video/pipw are undefined below.
  }

  idl_test(
    ['picture-in-picture'],
    ['html', 'dom'],
    idl_array => {
      idl_array.add_objects({
        Document: ['document'],
        DocumentOrShadowRoot: ['document'],
        HTMLVideoElement: ['video'],
        PictureInPictureWindow: ['pipw'],
      });
    },
    'picture-in-picture interfaces.');
})
