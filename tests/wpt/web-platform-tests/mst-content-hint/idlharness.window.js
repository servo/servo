// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/webrtc/RTCPeerConnection-helper.js

'use strict';

idl_test(
  ['mst-content-hint'],
  ['mediacapture-streams', 'webrtc', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      MediaStreamTrack: ['audioTrack', 'videoTrack'],
    });

    const stream = await getNoiseStream({ audio: true, video: true });
    self.audioTrack = stream.getAudioTracks()[0];
    self.videoTrack = stream.getVideoTracks()[0];
  }
);
