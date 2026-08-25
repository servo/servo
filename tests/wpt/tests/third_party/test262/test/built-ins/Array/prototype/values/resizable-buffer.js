// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.values
description: >
  Array.p.values behaves correctly on TypedArrays backed by resizable buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function IteratorToNumbers(iterator) {
  const result = [];
  for (let value of iterator) {
    result.push(Number(value));
  }
  return result;
}

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

  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(fixedLength)), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(fixedLengthWithOffset)), [
    4,
    6
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTracking)), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTrackingWithOffset)), [
    4,
    6
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  // TypedArray.prototype.{entries, keys, values} throw right away when
  // called. Array.prototype.{entries, keys, values} don't throw, but when
  // we try to iterate the returned ArrayIterator, that throws.
  Array.prototype.values.call(fixedLength);
  Array.prototype.values.call(fixedLengthWithOffset);

  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLength));
  });
  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLengthWithOffset));
  });
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTracking)), [
    0,
    2,
    4
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTrackingWithOffset)), [4]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  Array.prototype.values.call(fixedLength);
  Array.prototype.values.call(fixedLengthWithOffset);
  Array.prototype.values.call(lengthTrackingWithOffset);

  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLength));
  });
  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLengthWithOffset));
  });
  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(lengthTrackingWithOffset));
  });
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTracking)), [0]);

  // Shrink to zero.
  rab.resize(0);
  Array.prototype.values.call(fixedLength);
  Array.prototype.values.call(fixedLengthWithOffset);
  Array.prototype.values.call(lengthTrackingWithOffset);

  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLength));
  });
  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(fixedLengthWithOffset));
  });
  assert.throws(TypeError, () => {
    Array.from(Array.prototype.values.call(lengthTrackingWithOffset));
  });
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTracking)), []);

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

  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(fixedLength)), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(fixedLengthWithOffset)), [
    4,
    6
  ]);
  assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTracking)), [
    0,
    2,
    4,
    6,
    8,
    10
  ]);
                      assert.compareArray(IteratorToNumbers(Array.prototype.values.call(lengthTrackingWithOffset)), [
    4,
    6,
    8,
    10
  ]);
}
