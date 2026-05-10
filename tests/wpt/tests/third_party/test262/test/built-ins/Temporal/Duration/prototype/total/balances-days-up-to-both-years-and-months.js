// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Balances days up to both years and months.
features: [Temporal]
---*/

const twoYears = new Temporal.Duration(0, 11, 0, 396, 0, 0, 0, 0, 0, 0);
assert.sameValue(twoYears.total({
  unit: "years",
  relativeTo: new Temporal.PlainDate(2017, 1, 1)
}), 2);

// (Negative)
const twoYearsNegative = new Temporal.Duration(0, -11, 0, -396, 0, 0, 0, 0, 0, 0);
assert.sameValue(twoYearsNegative.total({
  unit: "years",
  relativeTo: new Temporal.PlainDate(2017, 1, 1)
}), -2);

