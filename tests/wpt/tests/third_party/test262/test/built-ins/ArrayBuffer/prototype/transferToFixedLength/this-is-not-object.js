// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfertofixedlength
description: Throws a TypeError if `this` valueis not an object.
info: |
  ArrayBuffer.prototype.transferToFixedLength ( [ newLength ] )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [arraybuffer-transfer, Symbol, BigInt]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.transferToFixedLength, "function");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(undefined);
}, "`this` value is undefined");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(null);
}, "`this` value is null");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(true);
}, "`this` value is Boolean");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call("");
}, "`this` value is String");

var symbol = Symbol();
assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(symbol);
}, "`this` value is Symbol");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(1);
}, "`this` value is Number");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transferToFixedLength.call(1n);
}, "`this` value is bigint");
