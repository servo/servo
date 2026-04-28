// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Balances up to the next unit after rounding.
features: [Temporal]
---*/

const almostWeek = new Temporal.Duration(0, 0, 0, 6, 20, 0, 0, 0, 0, 0);
const almostWeekNegative = new Temporal.Duration(0, 0, 0, -6, -20, 0, 0, 0, 0, 0);

const totalWeeks = almostWeek.total({
  unit: "weeks",
  relativeTo: new Temporal.PlainDate(2020, 1, 1)
});
assert(Math.abs(totalWeeks - (6 + 20 / 24) / 7) < Number.EPSILON);

const totalWeeksNegative = almostWeekNegative.total({
  unit: "weeks",
  relativeTo: new Temporal.PlainDate(2020, 1, 1)
});
assert(Math.abs(totalWeeksNegative - -((6 + 20 / 24) / 7)) < Number.EPSILON);
