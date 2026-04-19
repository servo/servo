// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.entries
description: >
  Array.p.entries behaves correctly when receiver is backed by a resizable
  buffer and resized mid-iteration
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

function ArrayEntriesHelper(ta) {
  return Array.prototype.entries.call(ta);
}

// Orig. array: [0, 2, 4, 6]
//              [0, 2, 4, 6] << fixedLength
//                    [4, 6] << fixedLengthWithOffset
//              [0, 2, 4, 6, ...] << lengthTracking
//                    [4, 6, ...] << lengthTrackingWithOffset

// Iterating with entries() (the 4 loops below).
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLength = new ctor(rab, 0, 4);
  // The fixed length array is not affected by resizing.
  TestIterationAndResize(ArrayEntriesHelper(fixedLength), [
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
    ],
    [
      3,
      6
    ]
  ], rab, 2, 6 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const fixedLengthWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT, 2);
  // The fixed length array is not affected by resizing.
  TestIterationAndResize(ArrayEntriesHelper(fixedLengthWithOffset), [
    [
      0,
      4
    ],
    [
      1,
      6
    ]
  ], rab, 2, 6 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTracking = new ctor(rab, 0);
  TestIterationAndResize(ArrayEntriesHelper(lengthTracking), [
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
    ],
    [
      3,
      6
    ],
    [
      4,
      0
    ],
    [
      5,
      0
    ]
  ], rab, 2, 6 * ctor.BYTES_PER_ELEMENT);
}
for (let ctor of ctors) {
  const rab = CreateRabForTest(ctor);
  const lengthTrackingWithOffset = new ctor(rab, 2 * ctor.BYTES_PER_ELEMENT);
  TestIterationAndResize(ArrayEntriesHelper(lengthTrackingWithOffset), [
    [
      0,
      4
    ],
    [
      1,
      6
    ],
    [
      2,
      0
    ],
    [
      3,
      0
    ]
  ], rab, 2, 6 * ctor.BYTES_PER_ELEMENT);
}
