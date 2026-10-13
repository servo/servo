// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-arraybuffer.prototype.immutable
description: Return value according to the [[ArrayBufferIsImmutable]] internal slot.
info: |
  get ArrayBuffer.prototype.immutable
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. Return IsImmutableBuffer(O).

  IsImmutableBuffer ( arrayBuffer )
  1. If arrayBuffer has an [[ArrayBufferIsImmutable]] internal slot, return true.
  2. Return false.
features: [ArrayBuffer, immutable-arraybuffer]
includes: [detachArrayBuffer.js]
---*/

var ab = new ArrayBuffer(1);
assert.sameValue(ab.immutable, false);

var iab = ab.transferToImmutable();
assert.sameValue(iab.immutable, true);

$DETACHBUFFER(ab);
assert.sameValue(ab.immutable, false);
