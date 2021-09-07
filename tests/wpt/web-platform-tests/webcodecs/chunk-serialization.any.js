// META: global=window
// META: script=/common/media.js
// META: script=/webcodecs/utils.js

var defaultAudioInit = {
  type: 'key',
  timestamp: 1234,
  duration: 9876,
  data: new Uint8Array([5, 6, 7, 8])
};

var defaultVideoInit = {
  type: 'key',
  timestamp: 1234,
  duration: 5678,
  data: new Uint8Array([9, 10, 11, 12])
};

function createDefaultChunk(type, init) {
  return type == 'audio' ? new EncodedAudioChunk(init) :
                           new EncodedVideoChunk(init);
}

function runTest(t, type) {
  let defaultInit = type == 'audio' ? defaultAudioInit : defaultVideoInit;
  let originalData = createDefaultChunk(type, defaultInit);

  let channel = new MessageChannel();
  let localPort = channel.port1;
  let externalPort = channel.port2;

  externalPort.onmessage = t.step_func((e) => {
    let newData = e.data;

    // We should have a valid deserialized buffer.
    assert_equals(newData.type, defaultInit.type, 'type');
    assert_equals(newData.duration, defaultInit.duration, 'duration');
    assert_equals(newData.timestamp, defaultInit.timestamp, 'timestamp');
    assert_equals(
        newData.byteLength, defaultInit.data.byteLength, 'byteLength');

    const originalData_copyDest = new Uint8Array(defaultInit.data);
    const newData_copyDest = new Uint8Array(defaultInit.data);

    originalData.copyTo(originalData_copyDest);
    newData.copyTo(newData_copyDest);

    for (var i = 0; i < newData_copyDest.length; ++i) {
      assert_equals(
          newData_copyDest[i], originalData_copyDest[i], `data (i=${i})`);
    }

    externalPort.postMessage('Done');
  })

  localPort.onmessage = t.step_func_done((e) => {
    assert_equals(originalData.type, defaultInit.type, 'type');
    assert_equals(originalData.duration, defaultInit.duration, 'duration');
    assert_equals(originalData.timestamp, defaultInit.timestamp, 'timestamp');
    assert_equals(
        originalData.byteLength, defaultInit.data.byteLength, 'byteLength');
  })

  localPort.postMessage(originalData);
}

async_test(t => {
  runTest(t, 'audio');
}, 'Verify EncodedAudioChunk is serializable.');


async_test(t => {
  runTest(t, 'video');
}, 'Verify EncodedVideoChunk is serializable.');
