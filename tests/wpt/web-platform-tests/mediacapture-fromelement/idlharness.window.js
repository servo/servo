// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediacapture-fromelement/

'use strict';

idl_test(
  ['mediacapture-fromelement'],
  ['mediacapture-streams', 'html', 'dom'],
  idl_array => {
    // Ignored errors will be surfaced when media/canvas undefined below.
    try {
      self.media = document.createElement('media');
      media.width = media.height = 10;
      document.body.appendChild(media);
    } catch (e) { }

    try {
      self.canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      canvas.width = canvas.height = 10;
      self.track = canvas.captureStream().getTracks()[0];
    } catch (e) { }

    idl_array.add_objects({
      HTMLMediaElement: ['media'],
      HTMLCanvasElement: ['canvas'],
      CanvasCaptureMediaStreamTrack: ['track'],
    });
  }
);
