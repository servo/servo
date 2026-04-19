// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reduceright
description: >
  Array.p.reduceRight behaves correctly on TypedArrays backed by resizable
  buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  // Write some data into the array.
  const taWrite = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, 2 * i);
  }

  // Orig. array: [0, 2, 4, 6]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, ...] << lengthTracking
  //                    [4, 6, ...] << lengthTrackingWithOffset

  function ReduceRightCollecting(array) {
    const reduceRightValues = [];
    Array.prototype.reduceRight.call(array, (acc, n) => {
      reduceRightValues.push(n);
    }, 'initial value');
    reduceRightValues.reverse();
    return ToNumbers(reduceRightValues);
  }
  assert.compareArray(ReduceRightCollecting(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ReduceRightCollecting(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(ReduceRightCollecting(lengthTracking), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ReduceRightCollecting(lengthTrackingWithOffset), [
    4,
    6
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  assert.compareArray(ReduceRightCollecting(fixedLength), []);
  assert.compareArray(ReduceRightCollecting(fixedLengthWithOffset), []);

  assert.compareArray(ReduceRightCollecting(lengthTracking), [
    0,
    2,
    4
  ]);
  assert.compareArray(ReduceRightCollecting(lengthTrackingWithOffset), [4]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.compareArray(ReduceRightCollecting(fixedLength), []);
  assert.compareArray(ReduceRightCollecting(fixedLengthWithOffset), []);

  assert.compareArray(ReduceRightCollecting(lengthTracking), [0]);

  // Shrink to zero.
  rab.resize(0);
  assert.compareArray(ReduceRightCollecting(fixedLength), []);
  assert.compareArray(ReduceRightCollecting(fixedLengthWithOffset), []);
  assert.compareArray(ReduceRightCollecting(lengthTrackingWithOffset), []);

  assert.compareArray(ReduceRightCollecting(lengthTracking), []);

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

  assert.compareArray(ReduceRightCollecting(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ReduceRightCollecting(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(ReduceRightCollecting(lengthTracking), [
    0,
    2,
    4,
    6,
    8,
    10
  ]);
  assert.compareArray(ReduceRightCollecting(lengthTrackingWithOffset), [
    4,
    6,
    8,
    10
  ]);
}
