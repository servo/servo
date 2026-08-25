// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Balances differently depending on relativeTo option
features: [Temporal]
---*/

const fortyDays = new Temporal.Duration(0, 0, 0, 40, 0, 0, 0, 0, 0, 0);
const negativeFortyDays = new Temporal.Duration(0, 0, 0, -40, 0, 0, 0, 0, 0, 0);

assert.sameValue(fortyDays.total({
  unit: "months",
  relativeTo: new Temporal.PlainDate(2020, 2, 1)
}).toPrecision(16), (1 + 11 / 31).toPrecision(16));
assert.sameValue(fortyDays.total({
  unit: "months",
  relativeTo: new Temporal.PlainDate(2020, 1, 1)
}).toPrecision(16), (1 + 9 / 29).toPrecision(16));

assert.sameValue(negativeFortyDays.total({
  unit: "months",
  relativeTo: new Temporal.PlainDate(2020, 3, 1)
}).toPrecision(16), (-(1 + 11 / 31)).toPrecision(16));
assert.sameValue(negativeFortyDays.total({
  unit: "months",
  relativeTo: new Temporal.PlainDate(2020, 4, 1)
}).toPrecision(16), (-(1 + 9 / 29)).toPrecision(16));
