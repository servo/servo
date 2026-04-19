// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Throws a TypeError if this does not have a [[DataView]] internal slot
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  2. If view does not have a [[DataView]] internal slot, throw a TypeError
  exception.
  ...
features: [DataView, ArrayBuffer, Int8Array, BigInt, arrow-function]
---*/

var getBigInt64 = DataView.prototype.getBigInt64;

assert.throws(TypeError, () => getBigInt64.call({}), "{}");

assert.throws(TypeError, () => getBigInt64.call([]), "[]");

var ab = new ArrayBuffer(1);
assert.throws(TypeError, () => getBigInt64.call(ab), "ArrayBuffer");

var ta = new Int8Array();
assert.throws(TypeError, () => getBigInt64.call(ta), "TypedArray");
