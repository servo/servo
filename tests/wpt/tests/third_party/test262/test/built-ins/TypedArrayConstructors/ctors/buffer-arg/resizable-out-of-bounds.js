// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializetypedarrayfromarraybuffer
description: >
  Creating a TypedArray from a resizable buffer with invalid bounds
  throw RangedError
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const rab = CreateResizableArrayBuffer(40, 80);
for (let ctor of ctors) {
  // Length too big.
  assert.throws(RangeError, () => {
    new ctor(rab, 0, 40 / ctor.BYTES_PER_ELEMENT + 1);
  });
  // Offset too close to the end.
  assert.throws(RangeError, () => {
    new ctor(rab, 40 - ctor.BYTES_PER_ELEMENT, 2);
  });
  // Offset beyond end.
  assert.throws(RangeError, () => {
    new ctor(rab, 40, 1);
  });
  if (ctor.BYTES_PER_ELEMENT > 1) {
    // Offset not a multiple of the byte size.
    assert.throws(RangeError, () => {
      new ctor(rab, 1, 1);
    });
    assert.throws(RangeError, () => {
      new ctor(rab, 1);
    });
  }
}
