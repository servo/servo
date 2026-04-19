// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
info: |
  ArrayBuffer.prototype.transfer ( [ newLength ] )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [arraybuffer-transfer]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.transfer, 'function');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer();
}, '`this` value is the ArrayBuffer prototype');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call({});
}, '`this` value is an object');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call([]);
}, '`this` value is an array');
