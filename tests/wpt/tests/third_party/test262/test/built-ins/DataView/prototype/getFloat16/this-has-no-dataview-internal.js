// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getfloat16
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
features: [Float16Array, Int8Array]
---*/

var getFloat16 = DataView.prototype.getFloat16;

assert.throws(TypeError, function() {
  getFloat16.call({});
}, "{}");

assert.throws(TypeError, function() {
  getFloat16.call([]);
}, "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, function() {
  getFloat16.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  getFloat16.call(ta);
}, "TypedArray");
