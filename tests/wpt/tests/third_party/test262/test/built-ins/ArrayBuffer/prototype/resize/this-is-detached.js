// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
info: |
  ArrayBuffer.prototype.resize ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  3. If IsSharedArrayBuffer(O) is true, throw a TypeError exception.
  4. If IsDetachedBuffer(O) is true, throw a TypeError exception.
  [...]
includes: [detachArrayBuffer.js]
features: [resizable-arraybuffer]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.resize, 'function');

var ab = new ArrayBuffer(1);

$DETACHBUFFER(ab);

assert.throws(TypeError, function() {
  ab.resize();
});
