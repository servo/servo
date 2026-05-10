// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.values
description: >
  TypedArray.p.values behaves correctly when receiver is backed by resizable
  buffer
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function ValuesFromTypedArrayEntries(ta) {
  let result = [];
  let expectedKey = 0;
  for (let [key, value] of ta.entries()) {
    assert.sameValue(key, expectedKey, 'TypedArray method .entries should return `expectedKey`.');
    ++expectedKey;
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

  assert.compareArray(ValuesFromTypedArrayEntries(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTracking), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTrackingWithOffset), [
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
  assert.throws(TypeError, () => {
    fixedLength.entries();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.entries();
  });

  assert.throws(TypeError, () => {
    Array.from(fixedLength.entries());
  });
  assert.throws(TypeError, () => {
    Array.from(fixedLengthWithOffset.entries());
  });
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTracking), [
    0,
    2,
    4
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTrackingWithOffset), [4]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.entries();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.entries();
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.entries();
  });

  assert.throws(TypeError, () => {
    Array.from(fixedLength.entries());
  });
  assert.throws(TypeError, () => {
    Array.from(fixedLengthWithOffset.entries());
  });
  assert.throws(TypeError, () => {
    Array.from(lengthTrackingWithOffset.entries());
  });
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTracking), [0]);

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    fixedLength.entries();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.entries();
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.entries();
  });

  assert.throws(TypeError, () => {
    Array.from(fixedLength.entries());
  });
  assert.throws(TypeError, () => {
    Array.from(fixedLengthWithOffset.entries());
  });
  assert.throws(TypeError, () => {
    Array.from(lengthTrackingWithOffset.entries());
  });
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTracking), []);

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

  assert.compareArray(ValuesFromTypedArrayEntries(fixedLength), [
    0,
    2,
    4,
    6
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(fixedLengthWithOffset), [
    4,
    6
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTracking), [
    0,
    2,
    4,
    6,
    8,
    10
  ]);
  assert.compareArray(ValuesFromTypedArrayEntries(lengthTrackingWithOffset), [
    4,
    6,
    8,
    10
  ]);
}
