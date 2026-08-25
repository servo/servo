// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isvalidintegerindex
description: >
  Getting and setting in-bounds and out-of-bounds indices on TypedArrays backed
  by resizable buffers.
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

for (let ctor of ctors) {
  const rab = CreateResizableArrayBuffer(4 * ctor.BYTES_PER_ELEMENT, 40 * ctor.BYTES_PER_ELEMENT);
  const array = new ctor(rab, 0, 4);
  // Initial values
  for (let i = 0; i < 4; ++i) {
    assert.sameValue(Convert(array[i]), 0);
  }
  // Within-bounds write
  for (let i = 0; i < 4; ++i) {
    array[i] = MayNeedBigInt(array, i);
  }
  // Within-bounds read
  for (let i = 0; i < 4; ++i) {
    assert.sameValue(Convert(array[i]), i, `${ctor} array fails within-bounds read`);
  }
  rab.resize(2 * ctor.BYTES_PER_ELEMENT);
  // OOB read. If the RAB isn't large enough to fit the entire TypedArray,
  // the length of the TypedArray is treated as 0.
  for (let i = 0; i < 4; ++i) {
    assert.sameValue(array[i], undefined);
  }
  // OOB write (has no effect)
  for (let i = 0; i < 4; ++i) {
    array[i] = MayNeedBigInt(array, 10);
  }
  rab.resize(4 * ctor.BYTES_PER_ELEMENT);
  // Within-bounds read
  for (let i = 0; i < 2; ++i) {
    assert.sameValue(array[i], MayNeedBigInt(array, i));
  }
  // The shrunk-and-regrown part got zeroed.
  for (let i = 2; i < 4; ++i) {
    assert.sameValue(array[i], MayNeedBigInt(array, 0));
  }
  rab.resize(40 * ctor.BYTES_PER_ELEMENT);
  // Within-bounds read
  for (let i = 0; i < 2; ++i) {
    assert.sameValue(array[i], MayNeedBigInt(array, i));
  }
  for (let i = 2; i < 4; ++i) {
    assert.sameValue(array[i], MayNeedBigInt(array, 0));
  }
}
