// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.getbiguint64
description: >
  Return abrupt from ToNumber(byteOffset)
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

assert.throws(Test262Error, () => sample.getBigUint64(bo1), "valueOf");

assert.throws(Test262Error, () => sample.getBigUint64(bo2), "toString");
