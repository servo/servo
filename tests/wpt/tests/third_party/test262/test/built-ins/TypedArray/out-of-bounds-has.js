// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isvalidintegerindex
description: >
  In-bound indices are testable with `in` on TypedArrays backed by resizable buffers.
info: |
  IsValidIntegerIndex ( O, index )
  ...
  6. Let length be IntegerIndexedObjectLength(O, getBufferByteLength).
  7. If length is out-of-bounds or ℝ(index) < 0 or ℝ(index) ≥ length, return false.
  ...
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 40 * ctor.BYTES_PER_ELEMENT);
  const array = new ctor(rab, 0, 4);
  // Within-bounds read
  for (let i = 0; i < 4; ++i) {
    assert(i in array);
  }
  rab.resize(2 * ctor.BYTES_PER_ELEMENT);
  // OOB read. If the RAB isn't large enough to fit the entire TypedArray,
  // the length of the TypedArray is treated as 0.
  for (let i = 0; i < 4; ++i) {
    assert(!(i in array));
  }
  rab.resize(4 * ctor.BYTES_PER_ELEMENT);
  // Within-bounds read
  for (let i = 0; i < 4; ++i) {
    assert(i in array);
  }
  rab.resize(40 * ctor.BYTES_PER_ELEMENT);
  // Within-bounds read
  for (let i = 0; i < 4; ++i) {
    assert(i in array);
  }
}
