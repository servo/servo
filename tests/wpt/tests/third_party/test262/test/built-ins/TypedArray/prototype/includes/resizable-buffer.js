// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.includes
description: >
  TypedArray.p.includes behaves correctly on TypedArrays backed by resizable
  buffers.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer, Array.prototype.includes]
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

  // If fixedLength is a BigInt array, they all are BigInt Arrays.
  let n2 = MayNeedBigInt(fixedLength, 2);
  let n4 = MayNeedBigInt(fixedLength, 4);

  assert(fixedLength.includes(n2));
  assert(!fixedLength.includes(undefined));
  assert(fixedLength.includes(n2, 1));
  assert(!fixedLength.includes(n2, 2));
  assert(fixedLength.includes(n2, -3));
  assert(!fixedLength.includes(n2, -2));
  assert(!fixedLengthWithOffset.includes(n2));
  assert(fixedLengthWithOffset.includes(n4));
  assert(!fixedLengthWithOffset.includes(undefined));
  assert(fixedLengthWithOffset.includes(n4, 0));
  assert(!fixedLengthWithOffset.includes(n4, 1));
  assert(fixedLengthWithOffset.includes(n4, -2));
  assert(!fixedLengthWithOffset.includes(n4, -1));
  assert(lengthTracking.includes(n2));
  assert(!lengthTracking.includes(undefined));
  assert(lengthTracking.includes(n2, 1));
  assert(!lengthTracking.includes(n2, 2));
  assert(lengthTracking.includes(n2, -3));
  assert(!lengthTracking.includes(n2, -2));
  assert(!lengthTrackingWithOffset.includes(n2));
  assert(lengthTrackingWithOffset.includes(n4));
  assert(!lengthTrackingWithOffset.includes(undefined));
  assert(lengthTrackingWithOffset.includes(n4, 0));
  assert(!lengthTrackingWithOffset.includes(n4, 1));
  assert(lengthTrackingWithOffset.includes(n4, -2));
  assert(!lengthTrackingWithOffset.includes(n4, -1));

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [0, 2, 4]
  //              [0, 2, 4, ...] << lengthTracking
  //                    [4, ...] << lengthTrackingWithOffset

  assert.throws(TypeError, () => {
    fixedLength.includes(n2);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.includes(n2);
  });

  assert(lengthTracking.includes(n2));
  assert(!lengthTracking.includes(undefined));
  assert(!lengthTrackingWithOffset.includes(n2));
  assert(lengthTrackingWithOffset.includes(n4));
  assert(!lengthTrackingWithOffset.includes(undefined));

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.includes(n2);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.includes(n2);
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.includes(n2);
  });

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    fixedLength.includes(n2);
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.includes(n2);
  });
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.includes(n2);
  });
  assert(!lengthTracking.includes(n2));
  assert(!lengthTracking.includes(undefined));

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

  let n8 = MayNeedBigInt(fixedLength, 8);

  assert(fixedLength.includes(n2));
  assert(!fixedLength.includes(undefined));
  assert(!fixedLength.includes(n8));
  assert(!fixedLengthWithOffset.includes(n2));
  assert(fixedLengthWithOffset.includes(n4));
  assert(!fixedLengthWithOffset.includes(undefined));
  assert(!fixedLengthWithOffset.includes(n8));
  assert(lengthTracking.includes(n2));
  assert(!lengthTracking.includes(undefined));
  assert(lengthTracking.includes(n8));
  assert(!lengthTrackingWithOffset.includes(n2));
  assert(lengthTrackingWithOffset.includes(n4));
  assert(!lengthTrackingWithOffset.includes(undefined));
  assert(lengthTrackingWithOffset.includes(n8));
}
