// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-%typedarray%.prototype
description: >
  TypedArrays that are backed by resizable buffers have the same prototypes
  as those backed by fixed-length buffers
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const rab = CreateResizableArrayBuffer(40, 80);
const ab = new ArrayBuffer(80);
for (let ctor of ctors) {
  const ta_rab = new ctor(rab, 0, 3);
  const ta_ab = new ctor(ab, 0, 3);
  assert.sameValue(ta_ab.__proto__, ta_rab.__proto__);
}
