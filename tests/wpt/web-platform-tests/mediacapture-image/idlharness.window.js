// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediacapture-image/

'use strict';

idl_test(
  ['image-capture'],
  ['mediacapture-streams', 'html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      ImageCapture : ['capture'],
      PhotoCapabilities: ['capabilities'],
    });

    const canvas = document.createElement('canvas');
    document.body.appendChild(canvas);
    const context = canvas.getContext("2d");
    context.fillStyle = "red";
    context.fillRect(0, 0, 10, 10);
    const track = canvas.captureStream().getVideoTracks()[0];
    self.capture = new ImageCapture(track);
    self.capabilities = await capture.getPhotoCapabilities();
  }
);
