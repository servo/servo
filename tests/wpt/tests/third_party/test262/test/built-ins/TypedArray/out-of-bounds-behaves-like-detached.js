// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-isvalidintegerindex
description: >
  TypedArrays backed by resizable buffers that are out-of-bounds behave
  as if they were detached
includes: [resizableArrayBufferUtils.js]
features: [resizable-arraybuffer]
---*/

const rab = CreateResizableArrayBuffer(16, 40);
const i8a = new Int8Array(rab, 0, 4);
i8a.__proto__ = { 2: 'wrong value' };
i8a[2] = 10;
assert.sameValue(i8a[2], 10);
assert(2 in i8a);
rab.resize(0);
assert.sameValue(i8a[2], undefined);
assert(!(2 in i8a));
