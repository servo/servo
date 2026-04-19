// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Calendar required when days = 0 but years/months/weeks non-zero
features: [Temporal]
---*/

const yearInstance = new Temporal.Duration(1999);
const monthInstance = new Temporal.Duration(0, 49);
const weekInstance = new Temporal.Duration(0, 0, 1);
const dayInstance = new Temporal.Duration(0, 0, 0, 42);

let relativeTo = new Temporal.PlainDate(2021, 12, 15);

assert.throws(
  RangeError,
  () => { yearInstance.total({ unit: "days" }); },
  "total a Duration with non-zero years fails without largest/smallest unit"
);
const yearResult = yearInstance.total({ unit: "days", relativeTo });
assert.sameValue(yearResult, 730120, "year duration contains proper days");

assert.throws(
  RangeError,
  () => { monthInstance.total({ unit: "days" }); },
  "total a Duration with non-zero month fails without largest/smallest unit"
);

const monthResult = monthInstance.total({ unit: "days", relativeTo });
assert.sameValue(monthResult, 1492, "month duration contains proper days");

assert.throws(
  RangeError,
  () => { weekInstance.total({ unit: "days" }); },
  "total a Duration with non-zero weeks fails without largest/smallest unit"
);

const weekResult = weekInstance.total({ unit: "days", relativeTo });
assert.sameValue(weekResult, 7, "week duration contains proper days");

const dayResultWithoutRelative = dayInstance.total({ unit: "days" });
const dayResultWithRelative = dayInstance.total({ unit: "days", relativeTo });
assert.sameValue(dayResultWithoutRelative, 42, "day duration without relative-to part contains proper days");
assert.sameValue(dayResultWithRelative, 42, "day duration with relative-to part contains proper days");
