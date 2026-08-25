// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  [...]
features: [SharedArrayBuffer, resizable-arraybuffer]
---*/

assert.sameValue(typeof SharedArrayBuffer.prototype.grow, 'function');

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow();
}, '`this` value is the SharedArrayBuffer prototype');

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call({});
}, '`this` value is an object');

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call([]);
}, '`this` value is an array');
