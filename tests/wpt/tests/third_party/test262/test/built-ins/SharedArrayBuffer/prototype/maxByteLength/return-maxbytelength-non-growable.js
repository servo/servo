// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.maxbytelength
description: Return value from [[ArrayBufferByteLength]] internal slot
info: |
  24.1.4.1 get SharedArrayBuffer.prototype.maxByteLength

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is false, throw a TypeError exception.
  4. If IsResizableArrayBuffer(O) is true, then
     [...]
  5. Else,
     a. Let length be O.[[ArrayBufferByteLength]].
  6. Return 𝔽(length).
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var ab1 = new SharedArrayBuffer(0);
assert.sameValue(ab1.maxByteLength, 0);

var ab2 = new SharedArrayBuffer(42);
assert.sameValue(ab2.maxByteLength, 42);
