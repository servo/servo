// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Ambiguous subtraction is handled according to the overflow option
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const mar31 = new Temporal.PlainDateTime(2020, 3, 31, 15, 0);

TemporalHelpers.assertPlainDateTime(
  mar31.subtract({ months: 1 }),
  2020, 2, "M02", 29, 15, 0, 0, 0, 0, 0,
  "constrain when ambiguous result (overflow options not supplied)"
);

TemporalHelpers.assertPlainDateTime(
  mar31.subtract({ months: 1 }, { overflow: "constrain" }),
  2020, 2, "M02", 29, 15, 0, 0, 0, 0, 0,
  "constrain when ambiguous result (overflow options supplied)"
);

assert.throws(
  RangeError,
  () => mar31.subtract({ months: 1 }, { overflow: "reject" }),
  "throw when ambiguous result with reject"
);
