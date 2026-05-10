// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-dataview.prototype.bytelength
description: Throws a TypeError exception when `this` is not Object
info: |
  24.2.4.2 get DataView.prototype.byteLength

  1. Let O be the this value.
  2. If Type(O) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var getter = Object.getOwnPropertyDescriptor(
  DataView.prototype, "byteLength"
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
