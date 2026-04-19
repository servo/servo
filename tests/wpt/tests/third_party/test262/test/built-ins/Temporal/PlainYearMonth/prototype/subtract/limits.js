// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: RangeError thrown when going out of range
features: [Temporal]
---*/

const min = Temporal.PlainYearMonth.from("-271821-04");
for (const overflow of ["reject", "constrain"]) {
  assert.throws(RangeError, () => min.subtract({ months: 1 }, { overflow }), overflow);
}
