// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Return abrupt from ToNumber(byteOffset)
features: [DataView, ArrayBuffer, BigInt]
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

assert.throws(Test262Error, function() {
  sample.setBigInt64(bo1, 1n);
}, "valueOf");

assert.throws(Test262Error, function() {
  sample.setBigInt64(bo2, 1n);
}, "toString");
