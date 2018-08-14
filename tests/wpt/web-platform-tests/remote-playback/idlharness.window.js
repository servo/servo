// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/common/media.js

'use strict';

// https://w3c.github.io/remoteplayback/

idl_test(
  ['remote-playback'],
  ['html', 'dom'],
  idl_array => {
    try {
      const media = document.createElement('video');
      media.src = getVideoURI('movie_5');
      media.width = media.height = 10;
      document.body.appendChild(media);
      self.media = media;
    } catch (e) {
      // Will be surfaced when media is undefined below.
    }

    idl_array.add_objects({
      HTMLVideoElement: ['media'],
      RemotePlayback: ['media.remote']
    });
  }
);
