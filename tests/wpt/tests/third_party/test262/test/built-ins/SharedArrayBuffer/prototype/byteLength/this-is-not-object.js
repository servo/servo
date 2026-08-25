// Copyright (C) 2016 the V8 project authors. All rights reserved.
// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Throws a TypeError exception when `this` is not Object
features: [SharedArrayBuffer, Symbol]
---*/

var getter = Object.getOwnPropertyDescriptor(
  SharedArrayBuffer.prototype, "byteLength"
).get;

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
