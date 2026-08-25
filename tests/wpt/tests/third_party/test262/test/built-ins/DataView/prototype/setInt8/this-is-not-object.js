// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setint8
description: Throws a TypeError if this is not Object
info: |
  24.2.4.15 DataView.prototype.setInt8 ( byteOffset, value )

  1. Let v be the this value.
  2. Return ? SetViewValue(v, byteOffset, true, "Int8", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  1. If Type(view) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var setInt8 = DataView.prototype.setInt8;

assert.throws(TypeError, function() {
  setInt8.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  setInt8.call(null);
}, "null");

assert.throws(TypeError, function() {
  setInt8.call(1);
}, "1");

assert.throws(TypeError, function() {
  setInt8.call("string");
}, "string");

assert.throws(TypeError, function() {
  setInt8.call(true);
}, "true");

assert.throws(TypeError, function() {
  setInt8.call(false);
}, "false");

var s = Symbol("1");
assert.throws(TypeError, function() {
  setInt8.call(s);
}, "symbol");
