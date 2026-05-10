// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint8
description: Throws a TypeError if this is not Object
info: |
  24.2.4.10 DataView.prototype.getUint8 ( byteOffset )

  1. Let v be the this value.
  2. Return ? GetViewValue(v, byteOffset, true, "Uint8").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

var getUint8 = DataView.prototype.getUint8;

assert.throws(TypeError, function() {
  getUint8.call(undefined);
}, "undefined");

assert.throws(TypeError, function() {
  getUint8.call(null);
}, "null");

assert.throws(TypeError, function() {
  getUint8.call(1);
}, "1");

assert.throws(TypeError, function() {
  getUint8.call("string");
}, "string");

assert.throws(TypeError, function() {
  getUint8.call(true);
}, "true");

assert.throws(TypeError, function() {
  getUint8.call(false);
}, "false");

var s = Symbol("1");
assert.throws(TypeError, function() {
  getUint8.call(s);
}, "symbol");
