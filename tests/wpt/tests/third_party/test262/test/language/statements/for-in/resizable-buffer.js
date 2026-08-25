// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer-length
description: >
  Indices of TypedArrays backed by resizable buffers are enumerable with
  for-in
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

let rab = CreateResizableArrayBuffer(100, 200);
for (let ctor of ctors) {
  const ta = new ctor(rab, 0, 3);
  let keys = '';
  for (const key in ta) {
    keys += key;
  }
  assert.sameValue(keys, '012');
}
