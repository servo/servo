// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getuint8
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
info: |
  24.2.4.10 DataView.prototype.getUint8 ( byteOffset )

  1. Let v be the this value.
  2. Return ? GetViewValue(v, byteOffset, true, "Uint8").

  24.2.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  2. If view does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
features: [Int8Array]
---*/

var getUint8 = DataView.prototype.getUint8;

assert.throws(TypeError, function() {
  getUint8.call({});
}, "{}");

assert.throws(TypeError, function() {
  getUint8.call([]);
}, "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, function() {
  getUint8.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  getUint8.call(ta);
}, "TypedArray");
