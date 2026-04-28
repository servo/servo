// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.add
description: Overflow option has no effect in ISO 8601 calendar
features: [Temporal]
---*/

const year1 = new Temporal.Duration(1);
const year1n = new Temporal.Duration(-1);
const month1 = new Temporal.Duration(0, 1);
const month1n = new Temporal.Duration(0, -1);

for (const year of [2023, 2024]) {
  for (const month of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]) {
    const yearmonth = new Temporal.PlainYearMonth(year, month);
    for (const duration of [year1, year1n, month1, month1n]) {
      const resultConstrain = yearmonth.add(duration, { overflow: "constrain" });
      const resultReject = yearmonth.add(duration, { overflow: "reject" });
      assert.sameValue(resultReject.year, resultConstrain.year, "year should be identical");
      assert.sameValue(resultReject.month, resultConstrain.month, "month should be identical");
      assert.sameValue(resultReject.toString(), resultConstrain.toString(), "toString should be identical");
    }
  }
}
