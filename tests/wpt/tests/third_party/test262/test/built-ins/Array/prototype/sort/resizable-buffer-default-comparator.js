// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.sort
description: >
  Array.p.sort behaves correctly on TypedArrays backed by resizable buffers.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]

---*/

// The default comparison function for Array.prototype.sort is the string sort.

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 8 * ctor.BYTES_PER_ELEMENT);
  const fixedLength = new ctor(rab, 0, 4);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  const lengthTracking = new ctor(rab, 0);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  const taFull = new ctor(rab, 0);
  function WriteUnsortedData() {
    // Write some data into the array.
    for (let i = 0; i < taFull.length; ++i) {
      taFull[i] = MayNeedBigInt(taFull, 10 - 2 * i);
    }
  }
  // Orig. array: [10, 8, 6, 4]
  //              [10, 8, 6, 4] << fixedLength
  //                     [6, 4] << fixedLengthWithOffset
  //              [10, 8, 6, 4, ...] << lengthTracking
  //                     [6, 4, ...] << lengthTrackingWithOffset

  WriteUnsortedData();
  Array.prototype.sort.call(fixedLength);
  assert.compareArray(ToNumbers(taFull), [
    10,
    4,
    6,
    8
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    4,
    6
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(lengthTracking);
  assert.compareArray(ToNumbers(taFull), [
    10,
    4,
    6,
    8
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    4,
    6
  ]);

  // Shrink so that fixed length TAs go out of bounds.
  rab.resize(3 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [10, 8, 6]
  //              [10, 8, 6, ...] << lengthTracking
  //                     [6, ...] << lengthTrackingWithOffset

  WriteUnsortedData();
  Array.prototype.sort.call(fixedLength);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    6
  ]);
  Array.prototype.sort.call(fixedLengthWithOffset);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    6
  ]);
  Array.prototype.sort.call(lengthTracking);
  assert.compareArray(ToNumbers(taFull), [
    10,
    6,
    8
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    6
  ]);

  // Shrink so that the TAs with offset go out of bounds.
  rab.resize(1 * ctor.BYTES_PER_ELEMENT);
  WriteUnsortedData();
  Array.prototype.sort.call(fixedLength);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), [10]);
  Array.prototype.sort.call(fixedLengthWithOffset);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), [10]);
  Array.prototype.sort.call(lengthTrackingWithOffset);   // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), [10]);
  Array.prototype.sort.call(lengthTracking);
  assert.compareArray(ToNumbers(taFull), [10]);

  // Shrink to zero.
  rab.resize(0);
  Array.prototype.sort.call(fixedLength);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), []);
  Array.prototype.sort.call(fixedLengthWithOffset);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), []);
  Array.prototype.sort.call(lengthTrackingWithOffset);  // OOB -> NOOP
  assert.compareArray(ToNumbers(taFull), []);
  Array.prototype.sort.call(lengthTracking);
  assert.compareArray(ToNumbers(taFull), []);

  // Grow so that all TAs are back in-bounds.
  rab.resize(6 * ctor.BYTES_PER_ELEMENT);

  // Orig. array: [10, 8, 6, 4, 2, 0]
  //              [10, 8, 6, 4] << fixedLength
  //                     [6, 4] << fixedLengthWithOffset
  //              [10, 8, 6, 4, 2, 0, ...] << lengthTracking
  //                     [6, 4, 2, 0, ...] << lengthTrackingWithOffset

  WriteUnsortedData();
  Array.prototype.sort.call(fixedLength);
  assert.compareArray(ToNumbers(taFull), [
    10,
    4,
    6,
    8,
    2,
    0
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(fixedLengthWithOffset);
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    4,
    6,
    2,
    0
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(lengthTracking);
  assert.compareArray(ToNumbers(taFull), [
    0,
    10,
    2,
    4,
    6,
    8
  ]);
  WriteUnsortedData();
  Array.prototype.sort.call(lengthTrackingWithOffset);
  assert.compareArray(ToNumbers(taFull), [
    10,
    8,
    0,
    2,
    4,
    6
  ]);
}
