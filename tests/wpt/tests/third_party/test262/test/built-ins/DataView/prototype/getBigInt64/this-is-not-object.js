// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: Throws a TypeError if this is not Object
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  1. If Type(view) is not Object, throw a TypeError exception.
  ...
features: [DataView, ArrayBuffer, Symbol, BigInt, arrow-function]
---*/

var getBigInt64 = DataView.prototype.getBigInt64;

assert.throws(TypeError, () => getBigInt64.call(undefined),
  "undefined");

assert.throws(TypeError, () => getBigInt64.call(null), "null");

assert.throws(TypeError, () => getBigInt64.call(1), "1");

assert.throws(TypeError, () => getBigInt64.call("string"), "string");

assert.throws(TypeError, () => getBigInt64.call(true), "true");

assert.throws(TypeError, () => getBigInt64.call(false), "false");

var s = Symbol("1");
assert.throws(TypeError, () => getBigInt64.call(s), "symbol");
