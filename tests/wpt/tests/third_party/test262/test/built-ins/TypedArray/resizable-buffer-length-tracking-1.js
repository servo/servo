// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  Basic functionality of length-tracking TypedArrays backed by resizable
  buffers
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const rab = CreateResizableArrayBuffer(16, 40);
let tas = [];
for (let ctor of ctors) {
  tas.push(new ctor(rab));
}
for (let ta of tas) {
  assert.sameValue(ta.length, 16 / ta.BYTES_PER_ELEMENT);
  assert.sameValue(ta.byteLength, 16);
}
rab.resize(40);
for (let ta of tas) {
  assert.sameValue(ta.length, 40 / ta.BYTES_PER_ELEMENT);
  assert.sameValue(ta.byteLength, 40);
}
// Resize to a number which is not a multiple of all byte_lengths.
rab.resize(19);
for (let ta of tas) {
  const expected_length = Math.floor(19 / ta.BYTES_PER_ELEMENT);
  assert.sameValue(ta.length, expected_length);
  assert.sameValue(ta.byteLength, expected_length * ta.BYTES_PER_ELEMENT);
}
rab.resize(1);
for (let ta of tas) {
  if (ta.BYTES_PER_ELEMENT == 1) {
    assert.sameValue(ta.length, 1);
    assert.sameValue(ta.byteLength, 1);
  } else {
    assert.sameValue(ta.length, 0);
    assert.sameValue(ta.byteLength, 0);
  }
}
rab.resize(0);
for (let ta of tas) {
  assert.sameValue(ta.length, 0);
  assert.sameValue(ta.byteLength, 0);
}
rab.resize(8);
for (let ta of tas) {
  assert.sameValue(ta.length, 8 / ta.BYTES_PER_ELEMENT);
  assert.sameValue(ta.byteLength, 8);
}
rab.resize(40);
for (let ta of tas) {
  assert.sameValue(ta.length, 40 / ta.BYTES_PER_ELEMENT);
  assert.sameValue(ta.byteLength, 40);
}
