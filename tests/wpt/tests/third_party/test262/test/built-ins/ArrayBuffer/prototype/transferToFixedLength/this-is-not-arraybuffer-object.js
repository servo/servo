// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: >
  Throws a TypeError if `this` does not have an [[ArrayBufferData]] internal slot.
info: |
  ArrayBuffer.prototype.transferToFixedLength ( [ newLength ] )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [arraybuffer-transfer]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.transferToFixedLength, 'function');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength();
}, '`this` value is the ArrayBuffer prototype');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call({});
}, '`this` value is an object');

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call([]);
}, '`this` value is an array');
