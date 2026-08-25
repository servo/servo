// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getint32
description: Throws a TypeError if this is not Object
info: |
  24.2.4.9 DataView.prototype.getInt32 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int32").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var getInt32 = DataView.prototype.getInt32;

assert.throws(TypeError, function() {
  getInt32.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  getInt32.call(null);
}, "null");

assert.throws(TypeError, function() {
  getInt32.call(1);
}, "1");

assert.throws(TypeError, function() {
  getInt32.call("string");
}, "string");

assert.throws(TypeError, function() {
  getInt32.call(true);
}, "true");

assert.throws(TypeError, function() {
  getInt32.call(false);
}, "false");

var s = Symbol("1");
assert.throws(TypeError, function() {
  getInt32.call(s);
}, "symbol");
