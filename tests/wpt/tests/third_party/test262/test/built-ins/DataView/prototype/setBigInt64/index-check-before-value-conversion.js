// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  RangeError exception for negative or non-integral index is thrown before
  the value conversion.
features: [DataView, ArrayBuffer, BigInt]
---*/

var dataView = new DataView(new ArrayBuffer(8), 0);

var poisoned = {
  valueOf() {
    throw new Test262Error("valueOf called");
  }
};

assert.throws(RangeError, function() {
  dataView.setBigInt64(-1.5, poisoned);
}, "setBigInt64(-1.5, poisoned)");

assert.throws(RangeError, function() {
  dataView.setBigInt64(-1, poisoned);
}, "setBigInt64(-1, poisoned)");

assert.throws(RangeError, function() {
  dataView.setBigInt64(-Infinity, poisoned);
}, "setBigInt64(-Infinity, poisoned)");

assert.throws(RangeError, function() {
  dataView.setBigInt64(Infinity, poisoned);
}, "setBigInt64(Infinity, poisoned)");
