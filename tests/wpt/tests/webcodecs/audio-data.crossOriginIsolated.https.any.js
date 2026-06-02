// META: global=window
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

var defaultInit = {
  timestamp: 1234,
  channels: 2,
  sampleRate: 8000,
  frames: 1,
};

function testAudioData(useView) {
  let localData =
      new SharedArrayBuffer(defaultInit.channels * defaultInit.frames * 4);
  let view = new Float32Array(localData);
  view[0] = -1.0;
  view[1] = 1.0;

  let audio_data_init = {
    timestamp: defaultInit.timestamp,
    data: useView ? view : localData,
    numberOfFrames: defaultInit.frames,
    numberOfChannels: defaultInit.channels,
    sampleRate: defaultInit.sampleRate,
    format: 'f32-planar',
  }

  let data = new AudioData(audio_data_init);

  let copyDest = new SharedArrayBuffer(data.allocationSize({planeIndex: 0}));
  let destView = new Float32Array(copyDest);
  data.copyTo(useView ? destView : copyDest, {planeIndex: 0});
  assert_equals(destView[0], -1.0, 'copyDest[0]');
  data.copyTo(useView ? destView : copyDest, {planeIndex: 1});
  assert_equals(destView[0], 1.0, 'copyDest[1]');
}

test(t => {
  testAudioData(/*useView=*/ false);
}, 'Test construction and copyTo() using a SharedArrayBuffer');

test(t => {
  testAudioData(/*useView=*/ true);
}, 'Test construction and copyTo() using a Uint8Array(SharedArrayBuffer)');
