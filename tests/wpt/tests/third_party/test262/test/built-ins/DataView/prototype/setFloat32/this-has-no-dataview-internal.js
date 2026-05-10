// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat32
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
info: |
  24.2.4.13 DataView.prototype.setFloat32 ( byteOffset, value [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be false.
  3. Return ? SetViewValue(v, byteOffset, littleEndian, "Float32", value).

  24.2.1.2 SetViewValue ( view, requestIndex, isLittleEndian, type, value )

  1. If Type(view) is not Object, throw a TypeError exception.
  2. If view does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
features: [Int8Array]
---*/

var setFloat32 = DataView.prototype.setFloat32;

assert.throws(TypeError, function() {
  setFloat32.call({});
}, "{}");

assert.throws(TypeError, function() {
  setFloat32.call([]);
}, "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, function() {
  setFloat32.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  setFloat32.call(ta);
}, "TypedArray");
