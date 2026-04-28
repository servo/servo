// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.maxbytelength
description: Returns 0 if the buffer is detached
info: |
  get ArrayBuffer.prototype.maxByteLength

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, return +0𝔽.
  [...]
includes: [detachArrayBuffer.js]
features: [resizable-arraybuffer]
---*/

var ab = new ArrayBuffer(1);

$DETACHBUFFER(ab);

assert.sameValue(ab.maxByteLength, 0);
