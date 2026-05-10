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
  [...]
features: [resizable-arraybuffer]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.resize, 'function');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.resize();
}, '`this` value is the ArrayBuffer prototype');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.resize.call({});
}, '`this` value is an object');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.resize.call([]);
}, '`this` value is an array');
