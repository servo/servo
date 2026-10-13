// META: global=window,dedicatedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

// https://w3c.github.io/media-source/

'use strict';

idl_test(
  ['media-source'],
  ['dom', 'html', 'url'],
  async idl_array => {
    // Setting up a SourceBuffer object in a worker needs a media element on the
    // main thread; only add objects in a Window. Interface exposure is checked
    // in both scopes.
    if (!GLOBAL.isWindow()) {
      return;
    }

    idl_array.add_objects({
      MediaSource: ['mediaSource'],
      SourceBuffer: ['sourceBuffer'],
      SourceBufferList: ['mediaSource.sourceBuffers'],
    });

    const video = document.createElement('video');
    self.mediaSource = new MediaSource();
    video.src = URL.createObjectURL(mediaSource);

    self.sourceBuffer = await new Promise((resolve, reject) => {
      mediaSource.addEventListener('sourceopen', () => {
        var defaultType = 'video/webm;codecs="vp8,vorbis"';
        if (MediaSource.isTypeSupported(defaultType)) {
          resolve(mediaSource.addSourceBuffer(defaultType));
        } else {
          resolve(mediaSource.addSourceBuffer('video/mp4'));
        }
      });
      step_timeout(() => reject(new Error('sourceopen event not fired')), 3000);
    });
  }
);
