// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediacapture-fromelement/

'use strict';

idl_test(
  ['mediacapture-fromelement'],
  ['mediacapture-streams', 'html', 'dom'],
  idl_array => {
    // Ignored errors will be surfaced when the elements are undefined below.
    try {
      self.video = document.createElement('video');
      video.width = video.height = 10;
      document.body.appendChild(video);
    } catch (e) { }

    try {
      self.audio = document.createElement('audio');
      document.body.appendChild(audio);
    } catch (e) { }

    try {
      self.canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      canvas.width = canvas.height = 10;
      self.track = canvas.captureStream().getTracks()[0];
    } catch (e) { }

    idl_array.add_objects({
      HTMLVideoElement: ['video'],
      HTMLAudioElement: ['audio'],
      HTMLCanvasElement: ['canvas'],
      CanvasCaptureMediaStreamTrack: ['track'],
    });
  }
);
