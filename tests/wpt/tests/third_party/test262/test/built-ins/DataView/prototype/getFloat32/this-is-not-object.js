// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat32
description: Throws a TypeError if this is not Object
info: |
  24.2.4.5 DataView.prototype.getFloat32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Float32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var getFloat32 = DataView.prototype.getFloat32;

assert.throws(TypeError, function() {
  getFloat32.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  getFloat32.call(null);
}, "null");

assert.throws(TypeError, function() {
  getFloat32.call(1);
}, "1");

assert.throws(TypeError, function() {
  getFloat32.call("string");
}, "string");

assert.throws(TypeError, function() {
  getFloat32.call(true);
}, "true");

assert.throws(TypeError, function() {
  getFloat32.call(false);
}, "false");

var s = Symbol("1");
assert.throws(TypeError, function() {
  getFloat32.call(s);
}, "symbol");
