// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.defineproperties
description: >
  Object.defineProperties behaves correctly on TypedArrays backed by
  resizable buffers
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function DefinePropertiesMayNeedBigInt(ta, index, value) {
  const values = {};
  values[index] = { value: MayNeedBigInt(ta, value) };
  Object.defineProperties(ta, values);
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const taFull = new ctor(rab, 0);

  // Orig. array: [0, 0, 0, 0]
  //              [0, 0, 0, 0] << fixedLength
  //                    [0, 0] << fixedLengthWithOffset
  //              [0, 0, 0, 0, ...] << lengthTracking
  //                    [0, 0, ...] << lengthTrackingWithOffset

  DefinePropertiesMayNeedBigInt(fixedLength, 0, 1);
  assert.compareArray(ToNumbers(taFull), [
    1,
    0,
    0,
    0
  ]);
  DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 0, 2);
  assert.compareArray(ToNumbers(taFull), [
    1,
    0,
    2,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTracking, 1, 3);
  assert.compareArray(ToNumbers(taFull), [
    1,
    3,
    2,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 1, 4);
  assert.compareArray(ToNumbers(taFull), [
    1,
    3,
    2,
    4
  ]);
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLength, 4, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 2, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(lengthTracking, 4, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 2, 8);
  });

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [1, 3, 2]
  //              [1, 3, 2, ...] << lengthTracking
  //                    [2, ...] << lengthTrackingWithOffset

  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLength, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 0, 8);
  });
  assert.compareArray(ToNumbers(taFull), [
    1,
    3,
    2
  ]);
  DefinePropertiesMayNeedBigInt(lengthTracking, 0, 5);
  assert.compareArray(ToNumbers(taFull), [
    5,
    3,
    2
  ]);
  DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 0, 6);
  assert.compareArray(ToNumbers(taFull), [
    5,
    3,
    6
  ]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLength, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 0, 8);
  });
  assert.compareArray(ToNumbers(taFull), [5]);
  DefinePropertiesMayNeedBigInt(lengthTracking, 0, 7);
  assert.compareArray(ToNumbers(taFull), [7]);

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLength, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(lengthTracking, 0, 8);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 0, 8);
  });
  assert.compareArray(ToNumbers(taFull), []);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  DefinePropertiesMayNeedBigInt(fixedLength, 0, 9);
  assert.compareArray(ToNumbers(taFull), [
    9,
    0,
    0,
    0,
    0,
    0
  ]);
  DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 0, 10);
  assert.compareArray(ToNumbers(taFull), [
    9,
    0,
    10,
    0,
    0,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTracking, 1, 11);
  assert.compareArray(ToNumbers(taFull), [
    9,
    11,
    10,
    0,
    0,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 2, 12);
  assert.compareArray(ToNumbers(taFull), [
    9,
    11,
    10,
    0,
    12,
    0
  ]);

  // Trying to define properties out of the fixed-length bounds throws.
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLength, 5, 13);
  });
  assert.throws(TypeError, () => {
    DefinePropertiesMayNeedBigInt(fixedLengthWithOffset, 3, 13);
  });
  assert.compareArray(ToNumbers(taFull), [
    9,
    11,
    10,
    0,
    12,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTracking, 4, 14);
  assert.compareArray(ToNumbers(taFull), [
    9,
    11,
    10,
    0,
    14,
    0
  ]);
  DefinePropertiesMayNeedBigInt(lengthTrackingWithOffset, 3, 15);
  assert.compareArray(ToNumbers(taFull), [
    9,
    11,
    10,
    0,
    14,
    15
  ]);
}
