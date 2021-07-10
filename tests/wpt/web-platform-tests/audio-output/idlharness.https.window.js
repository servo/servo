// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://w3c.github.io/mediacapture-output/

'use strict';

idl_test(
  ['audio-output'],
  ['mediacapture-streams', 'html', 'dom'],
  idl_array => {
    self.audio = document.createElement('audio');
    self.video = document.createElement('video');
    idl_array.add_objects({
      HTMLAudioElement: ['audio'],
      HTMLVideoElement: ['video'],
      MediaDevices: ['navigator.mediaDevices'],
    });
  }
);
