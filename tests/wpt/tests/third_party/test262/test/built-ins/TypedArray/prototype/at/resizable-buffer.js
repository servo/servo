// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.at
description: >
  TypedArray.p.at behaves correctly on TypedArrays backed by resizable buffers
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function TypedArrayAtHelper(ta, index) {
  const result = ta.at(index);
  return Convert(result);
}

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);

  // Write some data into the array.
  let ta_write = new ctor(rab);
  for (let i = 0; i < 4; ++i) {
    ta_write[i] = MayNeedBigInt(ta_write, i);
  }
  assert.sameValue(TypedArrayAtHelper(fixedLength, -1), 3);
  assert.sameValue(TypedArrayAtHelper(lengthTracking, -1), 3);
  assert.sameValue(TypedArrayAtHelper(fixedLengthWithOffset, -1), 3);
  assert.sameValue(TypedArrayAtHelper(lengthTrackingWithOffset, -1), 3);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    TypedArrayAtHelper(fixedLength, -1);
  });
  assert.throws(TypeError, () => {
    TypedArrayAtHelper(fixedLengthWithOffset, -1);
  });
  assert.sameValue(TypedArrayAtHelper(lengthTracking, -1), 2);
  assert.sameValue(TypedArrayAtHelper(lengthTrackingWithOffset, -1), 2);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    TypedArrayAtHelper(fixedLength, -1);
  });
  assert.throws(TypeError, () => {
    TypedArrayAtHelper(fixedLengthWithOffset, -1);
  });
  assert.throws(TypeError, () => {
    TypedArrayAtHelper(lengthTrackingWithOffset, -1);
  });
  assert.sameValue(TypedArrayAtHelper(lengthTracking, -1), 0);

  // Grow so that all TAs are back in-bounds. New memory is zeroed.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  assert.sameValue(TypedArrayAtHelper(fixedLength, -1), 0);
  assert.sameValue(TypedArrayAtHelper(lengthTracking, -1), 0);
  assert.sameValue(TypedArrayAtHelper(fixedLengthWithOffset, -1), 0);
  assert.sameValue(TypedArrayAtHelper(lengthTrackingWithOffset, -1), 0);
}
