// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tolocalestring
description: >
  Array.p.toLocaleString behaves correctly on TypedArrays backed by resizable
  buffers.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const taWrite = new ctor(rab);

  // toLocaleString separator is implementation dependent.
  function listToString(list) {
    const comma = ['',''].toLocaleString();
    return list.join(comma);
  }

  // Write some data into the array.
  for (let i = 0; i < 4; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, 2 * i);
  }

  // Orig. array: [0, 2, 4, 6]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, ...] << lengthTracking
  //                    [4, 6, ...] << lengthTrackingWithOffset

  assert.sameValue(Array.prototype.toLocaleString.call(fixedLength), listToString([0,2,4,6]));
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLengthWithOffset), listToString([4,6]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTracking), listToString([0,2,4,6]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTrackingWithOffset), listToString([4,6]));

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  assert.sameValue(Array.prototype.toLocaleString.call(fixedLength), listToString([]));
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLengthWithOffset), listToString([]));

  assert.sameValue(Array.prototype.toLocaleString.call(lengthTracking), listToString([0,2,4]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTrackingWithOffset), listToString([4]));

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLength), listToString([]));
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLengthWithOffset), listToString([]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTrackingWithOffset), listToString([]));

  assert.sameValue(Array.prototype.toLocaleString.call(lengthTracking), listToString([0]));

  // Shrink to zero.
  rab.resize(0);
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLength), listToString([]));
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLengthWithOffset), listToString([]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTrackingWithOffset), listToString([]));

  assert.sameValue(Array.prototype.toLocaleString.call(lengthTracking), listToString([]));

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, 2 * i);
  }

  // Orig. array: [0, 2, 4, 6, 8, 10]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, 8, 10, ...] << lengthTracking
  //                    [4, 6, 8, 10, ...] << lengthTrackingWithOffset

  assert.sameValue(Array.prototype.toLocaleString.call(fixedLength), listToString([0,2,4,6]));
  assert.sameValue(Array.prototype.toLocaleString.call(fixedLengthWithOffset), listToString([4,6]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTracking), listToString([0,2,4,6,8,10]));
  assert.sameValue(Array.prototype.toLocaleString.call(lengthTrackingWithOffset), listToString([4,6,8,10]));
}
