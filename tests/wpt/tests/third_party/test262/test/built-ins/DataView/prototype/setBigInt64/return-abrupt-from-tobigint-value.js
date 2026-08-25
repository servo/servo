// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Return abrupt from ToBigInt(value)
features: [DataView, ArrayBuffer, BigInt]
---*/

var buffer = new ArrayBuffer(8);
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
  sample.setBigInt64(0, bo1);
}, "valueOf");

assert.throws(Test262Error, function() {
  sample.setBigInt64(0, bo2);
}, "toString");
