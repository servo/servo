// Copyright (C) 2023 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-arraybuffer.prototype.detached
description: Returns true if the buffer is detached, else false
info: |
  get ArrayBuffer.prototype.detached

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. Return IsDetachedBuffer(O).

includes: [detachArrayBuffer.js]
features: [ArrayBuffer, arraybuffer-transfer]
---*/

var ab = new ArrayBuffer(1);

assert.sameValue(ab.detached, false);

$DETACHBUFFER(ab);

assert.sameValue(ab.detached, true);
