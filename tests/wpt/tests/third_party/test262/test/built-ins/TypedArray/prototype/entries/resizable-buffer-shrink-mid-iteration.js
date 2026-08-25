// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype.entries
description: >
  TypedArray.p.entries behaves correctly when receiver is backed by resizable
  buffer that is shrunk mid-iteration
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset
// Iterating with entries() (the 4 loops below).
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);

  // The fixed length array goes out of bounds when the RAB is resized.
  assert.throws(TypeError, () => {
    TestIterationAndResize(fixedLength.entries(), null, rab, 2, 3 * ctor.BYTES_PER_ELEMENT);
  });
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);

  // The fixed length array goes out of bounds when the RAB is resized.
  assert.throws(TypeError, () => {
    TestIterationAndResize(fixedLengthWithOffset.entries(), null, rab, 1, 3 * ctor.BYTES_PER_ELEMENT);
  });
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  TestIterationAndResize(lengthTracking.entries(), [
    [
      0,
      0
    ],
    [
      1,
      2
    ],
    [
      2,
      4
    ]
  ], rab, 2, 3 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  TestIterationAndResize(lengthTrackingWithOffset.entries(), [
    [
      0,
      4
    ],
    [
      1,
      6
    ]
  ], rab, 2, 3 * ctor.BYTES_PER_ELEMENT);
}
