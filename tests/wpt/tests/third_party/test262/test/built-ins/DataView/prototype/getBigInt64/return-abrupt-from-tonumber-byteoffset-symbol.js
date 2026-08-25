// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Return abrupt from ToNumber(symbol byteOffset)
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToNumber(requestIndex).
  ...
features: [DataView, ArrayBuffer, Symbol, BigInt, arrow-function]
---*/

var buffer = new ArrayBuffer(1);
var sample = new DataView(buffer, 0);

var s = Symbol("1");

assert.throws(TypeError, () => sample.getBigInt64(s));
