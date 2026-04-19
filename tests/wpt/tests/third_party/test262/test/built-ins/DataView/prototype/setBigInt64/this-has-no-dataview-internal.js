// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
features: [DataView, ArrayBuffer, BigInt]
---*/

var setBigInt64 = DataView.prototype.setBigInt64;

assert.throws(TypeError, function() {
  setBigInt64.call({});
}, "{}");

assert.throws(TypeError, function() {
  setBigInt64.call([]);
}, "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, function() {
  setBigInt64.call(ab);
}, "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, function() {
  setBigInt64.call(ta);
}, "TypedArray");
