// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferMaxByteLength]] internal slot.
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  [...]
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

var ab;

assert.sameValue(typeof SharedArrayBuffer.prototype.grow, 'function');

ab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.grow(0);
}, 'zero byte length');

ab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.grow(3);
}, 'smaller byte length');

ab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.grow(4);
}, 'same byte length');

ab = new SharedArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.grow(5);
}, 'larger byte length');
