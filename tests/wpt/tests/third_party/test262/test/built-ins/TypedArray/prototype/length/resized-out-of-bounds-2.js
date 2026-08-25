// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-%typedarray%.prototype.length
description: >
  TypedArray.p.length behaves correctly when the underlying resizable buffer is
  resized such that the TypedArray becomes out of bounds.
includes: [compareArray.js, resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

// Like resized-out-of-bounds-1.js but with offsets.

const rab = CreateResizableArrayBuffer(20, 40);

// Create TAs with offset, which cover the bytes 8-15.
let tas_and_lengths = [];
for (let ctor of ctors) {
  const length = 8 / ctor.BYTES_PER_ELEMENT;
  tas_and_lengths.push([
    new ctor(rab, 8, length),
    length
  ]);
}
for (let [ta, length] of tas_and_lengths) {
  assert.sameValue(ta.length, length);
}
rab.resize(10);
for (let [ta, length] of tas_and_lengths) {
  assert.sameValue(ta.length, 0);
}
// Resize the rab so that it just barely covers the needed 8 bytes.
rab.resize(16);
for (let [ta, length] of tas_and_lengths) {
  assert.sameValue(ta.length, length);
}
rab.resize(40);
for (let [ta, length] of tas_and_lengths) {
  assert.sameValue(ta.length, length);
}
