// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.maxbytelength
description: Return value from [[ArrayBufferMaxByteLength]] internal slot
info: |
  24.1.4.1 get ArrayBuffer.prototype.maxByteLength

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, return +0𝔽.
  5. If IsResizableArrayBuffer(O) is true, then
     a. Let length be O.[[ArrayBufferMaxByteLength]].
  6. Else,
     [...]
  7. Return 𝔽(length).
features: [resizable-arraybuffer]
---*/

var ab1 = new ArrayBuffer(0, { maxByteLength: 0 });
assert.sameValue(ab1.maxByteLength, 0);

var ab2 = new ArrayBuffer(0, { maxByteLength: 23 });
assert.sameValue(ab2.maxByteLength, 23);

var ab3 = new ArrayBuffer(42, { maxByteLength: 42 });
assert.sameValue(ab3.maxByteLength, 42);
