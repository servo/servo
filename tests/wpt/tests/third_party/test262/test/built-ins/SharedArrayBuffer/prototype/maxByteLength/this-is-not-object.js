// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-sharedarraybuffer.prototype.maxbytelength
description: Throws a TypeError exception when `this` is not Object
info: |
  get SharedArrayBuffer.prototype.maxByteLength

  1. Let O be the this value.
  2. Perform ? RequireInternalSlot(O, [[ArrayBufferData]]).
  [...]
features: [SharedArrayBuffer, Symbol, resizable-arraybuffer]
---*/

var getter = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "maxByteLength"
).get;

assert.sameValue(typeof getter, "function");

assert.throws(TypeError, function() {
  getter.call(undefined);
}, "this is undefined");

assert.throws(TypeError, function() {
  getter.call(null);
}, "this is null");

assert.throws(TypeError, function() {
  getter.call(42);
}, "this is 42");

assert.throws(TypeError, function() {
  getter.call("1");
}, "this is a string");

assert.throws(TypeError, function() {
  getter.call(true);
}, "this is true");

assert.throws(TypeError, function() {
  getter.call(false);
}, "this is false");

var s = Symbol("s");
assert.throws(TypeError, function() {
  getter.call(s);
}, "this is a Symbol");
