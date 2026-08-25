// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: Throws a TypeError if this is not Object
features: [Float16Array, Symbol]
---*/

var setFloat16 = DataView.prototype.setFloat16;

assert.throws(TypeError, function() {
  setFloat16.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  setFloat16.call(null);
}, "null");

assert.throws(TypeError, function() {
  setFloat16.call(1);
}, "1");

assert.throws(TypeError, function() {
  setFloat16.call("string");
}, "string");

assert.throws(TypeError, function() {
  setFloat16.call(true);
}, "true");

assert.throws(TypeError, function() {
  setFloat16.call(false);
}, "false");

var s = Symbol("1");
assert.throws(TypeError, function() {
  setFloat16.call(s);
}, "symbol");
