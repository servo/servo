// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://wicg.github.io/media-capabilities/

'use strict';

promise_test(async () => {
  try {
    const video = {
      contentType: 'video/webm; codecs="vp09.00.10.08"',
      width: 800,
      height: 600,
      bitrate: 3000,
      framerate: 24,
    };
    self.decodingInfo = await navigator.mediaCapabilities.decodingInfo({
      type: 'file',
      video: video,
    });
    self.encodingInfo = await navigator.mediaCapabilities.encodingInfo({
      type: 'record',
      video: video
    });
  } catch (e) {
    // Will be surfaced when encodingInfo/decodingInfo is undefined below.
  }

  idl_test(
    ['media-capabilities'],
    ['html', 'cssom-view'],
    idl_array => {
      if (self.GLOBAL.isWorker()) {
        idl_array.add_objects({ WorkerNavigator: ['navigator'] });
      } else {
        idl_array.add_objects({ Navigator: ['navigator'] });
      }
      idl_array.add_objects({
        MediaCapabilities: ['navigator.mediaCapabilities'],
        MediaCapabilitiesInfo: ['decodingInfo', 'encodingInfo'],
        Screen: ['screen'],
        ScreenLuminance: ['screen.luminance'],
      });
    }
  );
});
