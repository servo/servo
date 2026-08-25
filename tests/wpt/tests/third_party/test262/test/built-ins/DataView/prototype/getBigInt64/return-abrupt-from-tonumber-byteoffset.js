// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbigint64
description: >
  Return abrupt from ToNumber(byteOffset)
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToNumber(requestIndex).
  ...
features: [DataView, ArrayBuffer, BigInt, arrow-function]
---*/

var buffer = new ArrayBuffer(1);
var sample = new DataView(buffer, 0);

var bo1 = {
  valueOf() {
    throw new Test262Error();
  }
};
var bo2 = {
  toString() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => sample.getBigInt64(bo1), "valueOf");

assert.throws(Test262Error, () => sample.getBigInt64(bo2), "toString");
