// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
features: [DataView, ArrayBuffer, Int8Array, BigInt, arrow-function]
---*/

var getBigUint64 = DataView.prototype.getBigUint64;

assert.throws(TypeError, () => getBigUint64.call({}), "{}");

assert.throws(TypeError, () => getBigUint64.call([]), "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, () => getBigUint64.call(ab), "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, () => getBigUint64.call(ta), "TypedArray");
