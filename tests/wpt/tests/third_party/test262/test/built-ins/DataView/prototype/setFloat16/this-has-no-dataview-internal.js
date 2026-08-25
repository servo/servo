// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setfloat16
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
features: [Float16Array, Int8Array]
---*/

var setFloat16 = DataView.prototype.setFloat16;

assert.throws(TypeError, function() {
  setFloat16.call({});
}, "{}");

assert.throws(TypeError, function() {
  setFloat16.call([]);
}, "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, function() {
  setFloat16.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  setFloat16.call(ta);
}, "TypedArray");
