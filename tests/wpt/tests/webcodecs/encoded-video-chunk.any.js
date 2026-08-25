// META: global=window,dedicatedworker
// META: script=/webcodecs/utils.js

test(t => {
  let chunk = new EncodedVideoChunk({type: 'key',
                                     timestamp: 10,
                                     duration: 300,
                                     data: new Uint8Array([0x0A, 0x0B, 0x0C])});
  assert_equals(chunk.type, 'key', 'type');
  assert_equals(chunk.timestamp, 10, 'timestamp');
  assert_equals(chunk.duration, 300, 'duration');
  assert_equals(chunk.byteLength, 3, 'byteLength');
  let copyDest = new Uint8Array(3);
  chunk.copyTo(copyDest);
  assert_equals(copyDest[0], 0x0A, 'copyDest[0]');
  assert_equals(copyDest[1], 0x0B, 'copyDest[1]');
  assert_equals(copyDest[2], 0x0C, 'copyDest[2]');

  // Make another chunk with different values for good measure.
  chunk = new EncodedVideoChunk({type: 'delta',
                                 timestamp: 100,
                                 data: new Uint8Array([0x00, 0x01])});
  assert_equals(chunk.type, 'delta', 'type');
  assert_equals(chunk.timestamp, 100, 'timestamp');
  assert_equals(chunk.duration, null, 'duration');
  assert_equals(chunk.byteLength, 2, 'byteLength');
  copyDest = new Uint8Array(2);
  chunk.copyTo(copyDest);
  assert_equals(copyDest[0], 0x00, 'copyDest[0]');
  assert_equals(copyDest[1], 0x01, 'copyDest[1]');
}, 'Test we can construct an EncodedVideoChunk.');

test(t => {
  let chunk = new EncodedVideoChunk({type: 'delta',
                                     timestamp: 100,
                                     data: new Uint8Array([0x00, 0x01, 0x02])});
  assert_throws_js(
    TypeError,
    () => chunk.copyTo(new Uint8Array(2)), 'destination is not large enough');

  const detached = makeDetachedArrayBuffer();
  assert_throws_js(
    TypeError,
    () => chunk.copyTo(detached), 'destination is detached');
}, 'Test copyTo() exception if destiation invalid');

test(t => {
  let chunk = new EncodedVideoChunk({type: 'key',
                                     timestamp: 10,
                                     duration: 300,
                                     data: new Uint8Array()});
  assert_equals(chunk.byteLength, 0, 'byteLength');
  let copyDest = new Uint8Array();
  chunk.copyTo(copyDest);
  assert_equals(copyDest.length, 0, 'copyDest.length');
}, 'Test we can construct an zero-sized EncodedVideoChunk.');
