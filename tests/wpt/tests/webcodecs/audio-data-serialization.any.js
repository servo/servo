// META: global=window
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

var defaultInit = {
  timestamp: 1234,
  channels: 2,
  sampleRate: 8000,
  frames: 100,
}

function createDefaultAudioData() {
  return make_audio_data(defaultInit.timestamp,
                          defaultInit.channels,
                          defaultInit.sampleRate,
                          defaultInit.frames);
}

async_test(t => {
  let originalData = createDefaultAudioData();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func((e) => {
    let newData = e.data;

    // We should have a valid deserialized buffer.
    assert_equals(newData.numberOfFrames, defaultInit.frames, 'numberOfFrames');
    assert_equals(
        newData.numberOfChannels, defaultInit.channels, 'numberOfChannels');
    assert_equals(newData.sampleRate, defaultInit.sampleRate, 'sampleRate');

    const originalData_copyDest = new Float32Array(defaultInit.frames);
    const newData_copyDest = new Float32Array(defaultInit.frames);

    for (var channel = 0; channel < defaultInit.channels; channel++) {
      originalData.copyTo(originalData_copyDest, { planeIndex: channel});
      newData.copyTo(newData_copyDest, { planeIndex: channel});

      for (var i = 0; i < newData_copyDest.length; i+=10) {
        assert_equals(newData_copyDest[i], originalData_copyDest[i],
          "data (ch=" + channel + ", i=" + i + ")");
      }
    }

    newData.close();
    externalPort.postMessage("Done");
  })

  localPort.onmessage = t.step_func_done((e) => {
    assert_equals(originalData.numberOfFrames, defaultInit.frames);
    originalData.close();
  })

  localPort.postMessage(originalData);

}, 'Verify closing AudioData does not propagate accross contexts.');

async_test(t => {
  let data = createDefaultAudioData();

  let channel = new MessageChannel();
  let localPort = channel.port1;

  localPort.onmessage = t.unreached_func();

  data.close();

  assert_throws_dom("DataCloneError", () => {
    localPort.postMessage(data);
  });

  t.done();
}, 'Verify posting closed AudioData throws.');

async_test(t => {
  let localData = createDefaultAudioData();

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func_done((e) => {
    let externalData = e.data;
    assert_equals(externalData.numberOfFrames, defaultInit.frames);
    externalData.close();
  })

  localPort.postMessage(localData, [localData]);
  assert_not_equals(localData.numberOfFrames, defaultInit.frames);
}, 'Verify transferring audio data closes them.');