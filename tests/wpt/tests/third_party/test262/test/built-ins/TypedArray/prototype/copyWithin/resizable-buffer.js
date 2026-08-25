// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.copywithin
description: >
  TypedArray.p.copyWithin behaves correctly when the receiver is backed by
  resizable buffer
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
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }

  // Orig. array: [0, 1, 2, 3]
  //              [0, 1, 2, 3] << fixedLength
  //                    [2, 3] << fixedLengthWithOffset
  //              [0, 1, 2, 3, ...] << lengthTracking
  //                    [2, 3, ...] << lengthTrackingWithOffset

  fixedLength.copyWithin(0, 2);
  assert.compareArray(ToNumbers(fixedLength), [
    2,
    3,
    2,
    3
  ]);
  for (let i = 0; i < 4; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }
  fixedLengthWithOffset.copyWithin(0, 1);
  assert.compareArray(ToNumbers(fixedLengthWithOffset), [
    3,
    3
  ]);
  for (let i = 0; i < 4; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }
  lengthTracking.copyWithin(0, 2);
  assert.compareArray(ToNumbers(lengthTracking), [
    2,
    3,
    2,
    3
  ]);
  lengthTrackingWithOffset.copyWithin(0, 1);
  assert.compareArray(ToNumbers(lengthTrackingWithOffset), [
    3,
    3
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  for (let i = 0; i < 3; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }

  // Orig. array: [0, 1, 2]
  //              [0, 1, 2, ...] << lengthTracking
  //                    [2, ...] << lengthTrackingWithOffset

  assert.throws(TypeError, () => {
    fixedLength.copyWithin(0, 1);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.copyWithin(0, 1);
  });
  assert.compareArray(ToNumbers(lengthTracking), [
    0,
    1,
    2
  ]);
  lengthTracking.copyWithin(0, 1);
  assert.compareArray(ToNumbers(lengthTracking), [
    1,
    2,
    2
  ]);
  lengthTrackingWithOffset.copyWithin(0, 1);
  assert.compareArray(ToNumbers(lengthTrackingWithOffset), [2]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  taWrite[0] = MayNeedBigInt(taWrite, 0);
  assert.throws(TypeError, () => {
    fixedLength.copyWithin(0, 1, 1);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.copyWithin(0, 1, 1);
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.copyWithin(0, 1, 1);
  });
  assert.compareArray(ToNumbers(lengthTracking), [0]);
  lengthTracking.copyWithin(0, 0, 1);
  assert.compareArray(ToNumbers(lengthTracking), [0]);

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    fixedLength.copyWithin(0, 1, 1);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.copyWithin(0, 1, 1);
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.copyWithin(0, 1, 1);
  });
  assert.compareArray(ToNumbers(lengthTracking), []);
  lengthTracking.copyWithin(0, 0, 1);
  assert.compareArray(ToNumbers(lengthTracking), []);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }

  // Orig. array: [0, 1, 2, 3, 4, 5]
  //              [0, 1, 2, 3] << fixedLength
  //                    [2, 3] << fixedLengthWithOffset
  //              [0, 1, 2, 3, 4, 5, ...] << lengthTracking
  //                    [2, 3, 4, 5, ...] << lengthTrackingWithOffset

  fixedLength.copyWithin(0, 2);
  assert.compareArray(ToNumbers(fixedLength), [
    2,
    3,
    2,
    3
  ]);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }
  fixedLengthWithOffset.copyWithin(0, 1);
  assert.compareArray(ToNumbers(fixedLengthWithOffset), [
    3,
    3
  ]);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }

  //              [0, 1, 2, 3, 4, 5, ...] << lengthTracking
  //        target ^     ^ start
  lengthTracking.copyWithin(0, 2);
  assert.compareArray(ToNumbers(lengthTracking), [
    2,
    3,
    4,
    5,
    4,
    5
  ]);
  for (let i = 0; i < 6; ++i) {
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }

  //                    [2, 3, 4, 5, ...] << lengthTrackingWithOffset
  //              target ^  ^ start
  lengthTrackingWithOffset.copyWithin(0, 1);
  assert.compareArray(ToNumbers(lengthTrackingWithOffset), [
    3,
    4,
    5,
    5
  ]);
}
