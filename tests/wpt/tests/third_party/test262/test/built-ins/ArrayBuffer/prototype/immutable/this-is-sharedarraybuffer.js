// Copyright (C) 2025 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-arraybuffer.prototype.immutable
description: Throws a TypeError exception when `this` is a SharedArrayBuffer
info: |
  get ArrayBuffer.prototype.immutable
  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
features: [SharedArrayBuffer, ArrayBuffer, immutable-arraybuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  ArrayBuffer.prototype, "immutable"
).get;

assert.sameValue(typeof getter, "function", "Getter must exist.");

var sab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  getter.call(sab);
});
