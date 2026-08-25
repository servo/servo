// META: global=window,dedicatedworker
// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=./utils.js
// META: timeout=long

'use strict';

var defaultCodecInit = {
  output: function() {
    assert_unreached("unexpected output");
  },
  error: function() {
    assert_unreached("unexpected error");
  },
}

var defaultAudioChunkInit = {
  type: 'key',
  timestamp: 1234,
  duration: 9876,
  data: new Uint8Array([5, 6, 7, 8])
};

var defaultVideoChunkInit = {
  type: 'key',
  timestamp: 1234,
  duration: 5678,
  data: new Uint8Array([9, 10, 11, 12])
};

idl_test(['webcodecs'], ['dom', 'html', 'webidl'], async idlArray => {
  self.imageBody =
      await fetch('four-colors.png').then(response => response.arrayBuffer());

  let decoder = new ImageDecoder({data: self.imageBody, type: 'image/png'});
  await decoder.tracks.ready;
  self.imageTracks = decoder.tracks.selectedTrack;

  idlArray.add_objects({
    AudioDecoder: [`new AudioDecoder(defaultCodecInit)`],
    VideoDecoder: [`new VideoDecoder(defaultCodecInit)`],
    AudioEncoder: [`new AudioEncoder(defaultCodecInit)`],
    VideoEncoder: [`new VideoEncoder(defaultCodecInit)`],
    EncodedAudioChunk: [`new EncodedAudioChunk(defaultAudioChunkInit)`],
    EncodedVideoChunk: [`new EncodedVideoChunk(defaultVideoChunkInit)`],
    AudioData: [`make_audio_data(1234, 2, 8000, 100)`],
    VideoFrame: [
      `new VideoFrame(makeImageBitmap(32, 16), {timestamp: 100, duration: 33})`
    ],
    VideoColorSpace: [
      `new VideoColorSpace()`,
      `new VideoColorSpace({primaries: 'bt709', transfer: 'bt709', matrix: 'bt709', fullRange: true})`,
    ],
    ImageDecoder:
        [`new ImageDecoder({data: self.imageBody, type: 'image/png'})`],
    ImageTrackList:
        [`new ImageDecoder({data: self.imageBody, type: 'image/png'}).tracks`],
    ImageTrack: [`self.imageTracks`],
  });
});
