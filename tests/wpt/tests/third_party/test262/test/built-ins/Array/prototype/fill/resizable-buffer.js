// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.fill
description: >
  Array.p.fill behaves correctly when the receiver is backed by
  resizable buffer
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function ReadDataFromBuffer(ab, ctor) {
  let result = [];
  const ta = new ctor(ab, 0, ab.byteLength / ctor.BYTES_PER_ELEMENT);
  for (let item of ta) {
    result.push(Number(item));
  }
  return result;
}

function ArrayFillHelper(ta, n, start, end) {
  if (ta instanceof BigInt64Array || ta instanceof BigUint64Array) {
    Array.prototype.fill.call(ta, BigInt(n), start, end);
  } else {
    Array.prototype.fill.call(ta, n, start, end);
  }
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    0,
    0,
    0,
    0
  ]);
  ArrayFillHelper(fixedLength, 1);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    1,
    1,
    1,
    1
  ]);
  ArrayFillHelper(fixedLengthWithOffset, 2);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    1,
    1,
    2,
    2
  ]);
  ArrayFillHelper(lengthTracking, 3);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    3,
    3,
    3,
    3
  ]);
  ArrayFillHelper(lengthTrackingWithOffset, 4);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    3,
    3,
    4,
    4
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  ArrayFillHelper(fixedLength, 5);
  ArrayFillHelper(fixedLengthWithOffset, 6);
  // We'll check below that these were no-op.

  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    3,
    3,
    4
  ]);
  ArrayFillHelper(lengthTracking, 7);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    7,
    7,
    7
  ]);
  ArrayFillHelper(lengthTrackingWithOffset, 8);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    7,
    7,
    8
  ]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  // We'll check below that these were no-op.
  ArrayFillHelper(fixedLength, 9);
  ArrayFillHelper(fixedLengthWithOffset, 10);
  ArrayFillHelper(lengthTrackingWithOffset, 11);

  assert.compareArray(ReadDataFromBuffer(rab, ctor), [7]);
  ArrayFillHelper(lengthTracking, 12);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [12]);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  ArrayFillHelper(fixedLength, 13);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    13,
    13,
    13,
    13,
    0,
    0
  ]);
  ArrayFillHelper(fixedLengthWithOffset, 14);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    13,
    13,
    14,
    14,
    0,
    0
  ]);
  ArrayFillHelper(lengthTracking, 15);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    15,
    15,
    15,
    15,
    15
  ]);
  ArrayFillHelper(lengthTrackingWithOffset, 16);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    15,
    16,
    16,
    16,
    16
  ]);

  // Filling with non-undefined start & end.
  ArrayFillHelper(fixedLength, 17, 1, 3);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    17,
    17,
    16,
    16,
    16
  ]);
  ArrayFillHelper(fixedLengthWithOffset, 18, 1, 2);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    17,
    17,
    18,
    16,
    16
  ]);
  ArrayFillHelper(lengthTracking, 19, 1, 3);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    19,
    19,
    18,
    16,
    16
  ]);
  ArrayFillHelper(lengthTrackingWithOffset, 20, 1, 2);
  assert.compareArray(ReadDataFromBuffer(rab, ctor), [
    15,
    19,
    19,
    20,
    16,
    16
  ]);
}
