// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-sharedarraybuffer.prototype.grow
description: Throws a TypeError if `this` valueis not an object.
info: |
  SharedArrayBuffer.prototype.grow ( newLength )

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferMaxByteLength]]).
  [...]
features: [BigInt, SharedArrayBuffer, Symbol, resizable-arraybuffer]
---*/

assert.sameValue(typeof SharedArrayBuffer.prototype.grow, "function");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(undefined);
}, "`this` value is undefined");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(null);
}, "`this` value is null");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(true);
}, "`this` value is Boolean");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call("");
}, "`this` value is String");

var symbol = Symbol();
assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(symbol);
}, "`this` value is Symbol");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(1);
}, "`this` value is Number");

assert.throws(TypeError, function() {
  SharedArrayBuffer.prototype.grow.call(1n);
}, "`this` value is bigint");
