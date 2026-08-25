// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.slice
description: >
  TypedArray.p.slice behaves correctly on TypedArrays backed by resizable
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
    taWrite[i] = MayNeedBigInt(taWrite, i);
  }
  const fixedLengthSlice = fixedLength.slice();
  assert.compareArray(ToNumbers(fixedLengthSlice), [
    0,
    1,
    2,
    3
  ]);
  assert(!fixedLengthSlice.buffer.resizable);
  const fixedLengthWithOffsetSlice = fixedLengthWithOffset.slice();
  assert.compareArray(ToNumbers(fixedLengthWithOffsetSlice), [
    2,
    3
  ]);
  assert(!fixedLengthWithOffsetSlice.buffer.resizable);
  const lengthTrackingSlice = lengthTracking.slice();
  assert.compareArray(ToNumbers(lengthTrackingSlice), [
    0,
    1,
    2,
    3
  ]);
  assert(!lengthTrackingSlice.buffer.resizable);
  const lengthTrackingWithOffsetSlice = lengthTrackingWithOffset.slice();
  assert.compareArray(ToNumbers(lengthTrackingWithOffsetSlice), [
    2,
    3
  ]);
  assert(!lengthTrackingWithOffsetSlice.buffer.resizable);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.slice();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.slice();
  });
  assert.compareArray(ToNumbers(lengthTracking.slice()), [
    0,
    1,
    2
  ]);
  assert.compareArray(ToNumbers(lengthTrackingWithOffset.slice()), [2]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  assert.throws(TypeError, () => {
    fixedLength.slice();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.slice();
  });
  assert.compareArray(ToNumbers(lengthTracking.slice()), [0]);
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.slice();
  });

  // Shrink to zero.
  rab.resize(0);
  assert.throws(TypeError, () => {
    fixedLength.slice();
  });
  assert.throws(TypeError, () => {
    fixedLengthWithOffset.slice();
  });
  assert.compareArray(ToNumbers(lengthTracking.slice()), []);
  assert.throws(TypeError, () => {
    lengthTrackingWithOffset.slice();
  });

  // Verify that the previously created slices aren't affected by the
  // shrinking.
  assert.compareArray(ToNumbers(fixedLengthSlice), [
    0,
    1,
    2,
    3
  ]);
  assert.compareArray(ToNumbers(fixedLengthWithOffsetSlice), [
    2,
    3
  ]);
  assert.compareArray(ToNumbers(lengthTrackingSlice), [
    0,
    1,
    2,
    3
  ]);
  assert.compareArray(ToNumbers(lengthTrackingWithOffsetSlice), [
    2,
    3
  ]);

  // Grow so that all TAs are back in-bounds. New memory is zeroed.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);
  assert.compareArray(ToNumbers(fixedLength.slice()), [
    0,
    0,
    0,
    0
  ]);
  assert.compareArray(ToNumbers(fixedLengthWithOffset.slice()), [
    0,
    0
  ]);
  assert.compareArray(ToNumbers(lengthTracking.slice()), [
    0,
    0,
    0,
    0,
    0,
    0
  ]);
  assert.compareArray(ToNumbers(lengthTrackingWithOffset.slice()), [
    0,
    0,
    0,
    0
  ]);
}
