// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-dataview.prototype.setbigint64
description: >
  Index bounds checks are performed after value conversion.
features: [DataView, ArrayBuffer, BigInt]
---*/

var dataView = new DataView(new ArrayBuffer(8), 0);

var poisoned = {
  valueOf() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  dataView.setBigInt64(100, poisoned);
}, "setBigInt64(100, poisoned)");

assert.throws(Test262Error, function() {
  dataView.setBigInt64('100', poisoned);
}, "setBigInt64('100', poisoned)");
