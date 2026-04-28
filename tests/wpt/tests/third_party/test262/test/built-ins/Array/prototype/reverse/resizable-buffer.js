// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.reverse
description: >
  Array.p.reverse behaves correctly on TypedArrays backed by resizable buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const wholeArrayView = new ctor(rab);
  function WriteData() {
    // Write some data into the array.
    for (let i = 0; i < wholeArrayView.length; ++i) {
      wholeArrayView[i] = MayNeedBigInt(wholeArrayView, 2 * i);
    }
  }
  WriteData();

  // Orig. array: [0, 2, 4, 6]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, ...] << lengthTracking
  //                    [4, 6, ...] << lengthTrackingWithOffset

  Array.prototype.reverse.call(fixedLength);
  assert.compareArray(ToNumbers(wholeArrayView), [
    6,
    4,
    2,
    0
  ]);
  Array.prototype.reverse.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    6,
    4,
    0,
    2
  ]);
  Array.prototype.reverse.call(lengthTracking);
  assert.compareArray(ToNumbers(wholeArrayView), [
    2,
    0,
    4,
    6
  ]);
  Array.prototype.reverse.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    2,
    0,
    6,
    4
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  WriteData();

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  Array.prototype.reverse.call(fixedLength);
  assert.compareArray(ToNumbers(wholeArrayView), [
    0,
    2,
    4
  ]);
  Array.prototype.reverse.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    0,
    2,
    4
  ]);
  Array.prototype.reverse.call(lengthTracking);
  assert.compareArray(ToNumbers(wholeArrayView), [
    4,
    2,
    0
  ]);
  Array.prototype.reverse.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    4,
    2,
    0
  ]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  WriteData();
  Array.prototype.reverse.call(fixedLength);
  assert.compareArray(ToNumbers(wholeArrayView), [0]);
  Array.prototype.reverse.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [0]);
  Array.prototype.reverse.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [0]);
  Array.prototype.reverse.call(lengthTracking);
  assert.compareArray(ToNumbers(wholeArrayView), [0]);

  // Shrink to zero.
  rab.resize(0);
  Array.prototype.reverse.call(fixedLength);
  assert.compareArray(ToNumbers(wholeArrayView), []);
  Array.prototype.reverse.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), []);
  Array.prototype.reverse.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), []);
  Array.prototype.reverse.call(lengthTracking);
  assert.compareArray(ToNumbers(wholeArrayView), []);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  WriteData();

  // Orig. array: [0, 2, 4, 6, 8, 10]
  //              [0, 2, 4, 6] << fixedLength
  //                    [4, 6] << fixedLengthWithOffset
  //              [0, 2, 4, 6, 8, 10, ...] << lengthTracking
  //                    [4, 6, 8, 10, ...] << lengthTrackingWithOffset

  Array.prototype.reverse.call(fixedLength);
  assert.compareArray(ToNumbers(wholeArrayView), [
    6,
    4,
    2,
    0,
    8,
    10
  ]);
  Array.prototype.reverse.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    6,
    4,
    0,
    2,
    8,
    10
  ]);
  Array.prototype.reverse.call(lengthTracking);
  assert.compareArray(ToNumbers(wholeArrayView), [
    10,
    8,
    2,
    0,
    4,
    6
  ]);
  Array.prototype.reverse.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(wholeArrayView), [
    10,
    8,
    6,
    4,
    0,
    2
  ]);
}
