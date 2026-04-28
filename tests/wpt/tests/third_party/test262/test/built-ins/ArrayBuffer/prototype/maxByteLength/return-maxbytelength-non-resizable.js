// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.maxbytelength
description: Return value from [[ArrayBufferByteLength]] internal slot
info: |
  24.1.4.1 get ArrayBuffer.prototype.maxByteLength

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, return +0𝔽.
  5. If IsResizableArrayBuffer(O) is true, then
     [...]
  6. Else,
     a. Let length be O.[[ArrayBufferByteLength]].
  7. Return 𝔽(length).
features: [resizable-arraybuffer]
---*/

var ab1 = new ArrayBuffer(0);
assert.sameValue(ab1.maxByteLength, 0);

var ab2 = new ArrayBuffer(42);
assert.sameValue(ab2.maxByteLength, 42);
