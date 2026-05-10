// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.resize
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferMaxByteLength]] internal slot.
info: |
  ArrayBuffer.prototype.resize ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  [...]
features: [resizable-arraybuffer]
---*/

var ab;

assert.sameValue(typeof ArrayBuffer.prototype.resize, 'function');

ab = new ArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.resize(0);
}, 'zero byte length');

ab = new ArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.resize(3);
}, 'smaller byte length');

ab = new ArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.resize(4);
}, 'same byte length');

ab = new ArrayBuffer(4);
assert.throws(TypeError, function() {
  ab.resize(5);
}, 'larger byte length');
