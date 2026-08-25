// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Throws a RangeError if getIndex < 0
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);

assert.throws(RangeError, () => sample.getBigInt64(-1),
  "DataView access at index -1 should throw");

assert.throws(RangeError, () => sample.getBigInt64(-Infinity),
  "DataView access at index -Infinity should throw");
