// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arraybuffer.prototype.transfer
description: Throws a TypeError if `this` valueis not an object.
info: |
  ArrayBuffer.prototype.transfer ( [ newLength ] )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [arraybuffer-transfer, Symbol, BigInt]
---*/

assert.sameValue(typeof ArrayBuffer.prototype.transfer, "function");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(undefined);
}, "`this` value is undefined");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(null);
}, "`this` value is null");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(true);
}, "`this` value is Boolean");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call("");
}, "`this` value is String");

var symbol = Symbol();
assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(symbol);
}, "`this` value is Symbol");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(1);
}, "`this` value is Number");

assert.throws(TypeError, function() {
  ArrayBuffer.prototype.transfer.call(1n);
}, "`this` value is bigint");
